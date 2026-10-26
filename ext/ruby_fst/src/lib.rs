use std::cell::RefCell;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use fst::automaton::{Automaton, Levenshtein};
use fst::raw::{CompiledAddr, Fst, Node, Output, Stream};
use fst::{IntoStreamer, Streamer};
use magnus::value::Lazy;
use magnus::{
    DataTypeFunctions, Error, ExceptionClass, RModule, RString, Ruby, TypedData, Value, function,
    method, prelude::*,
};
use memmap2::Mmap;

// ---------------------------------------------------------------------------
// Errors
//
// I/O failures raise IOError, a rejected Levenshtein query raises
// ArgumentError, and everything else (invalid or corrupt FST data, misuse of
// a builder) raises RubyFst::Error, a RuntimeError subclass.
// ---------------------------------------------------------------------------

static FST_ERROR: Lazy<ExceptionClass> = Lazy::new(|ruby| {
    ruby.class_object()
        .const_get::<_, RModule>("RubyFst")
        .and_then(|module| module.const_get("Error"))
        .expect("RubyFst::Error is defined in init")
});

fn fst_error(ruby: &Ruby, msg: impl std::fmt::Display) -> Error {
    Error::new(ruby.get_inner(&FST_ERROR), msg.to_string())
}

fn io_error(ruby: &Ruby, path: &Path, err: io::Error) -> Error {
    Error::new(
        ruby.exception_io_error(),
        format!("{}: {err}", path.display()),
    )
}

fn map_fst_error(ruby: &Ruby, err: fst::Error) -> Error {
    match err {
        fst::Error::Io(err) => Error::new(ruby.exception_io_error(), err.to_string()),
        fst::Error::Fst(err) => fst_error(ruby, err),
    }
}

// ---------------------------------------------------------------------------
// Storage: the backing buffer for an FST, either an owned heap allocation or a
// memory map. A heap buffer is reported to Ruby's GC so that large FSTs count
// towards its memory pressure; a map is backed by the file, so it isn't.
// ---------------------------------------------------------------------------

enum Storage {
    Mem(Vec<u8>),
    Mmap(Mmap),
}

impl Storage {
    fn mem(ruby: &Ruby, bytes: Vec<u8>) -> Self {
        ruby.gc_adjust_memory_usage(heap_size(&bytes));
        Storage::Mem(bytes)
    }

    fn read(ruby: &Ruby, path: &Path) -> Result<Self, Error> {
        let bytes = fs::read(path).map_err(|e| io_error(ruby, path, e))?;
        Ok(Self::mem(ruby, bytes))
    }

    fn mmap(ruby: &Ruby, path: &Path) -> Result<Self, Error> {
        let file = File::open(path).map_err(|e| io_error(ruby, path, e))?;
        // SAFETY: modifying or truncating a mapped file is undefined behavior,
        // and nothing here can prevent another process from doing it. The
        // README makes keeping the file unchanged the caller's responsibility,
        // and `save` replaces files by rename, so it never truncates a mapping.
        let map = unsafe { Mmap::map(&file) }.map_err(|e| io_error(ruby, path, e))?;
        Ok(Storage::Mmap(map))
    }
}

impl AsRef<[u8]> for Storage {
    fn as_ref(&self) -> &[u8] {
        match self {
            Storage::Mem(bytes) => bytes,
            Storage::Mmap(map) => map,
        }
    }
}

impl Drop for Storage {
    fn drop(&mut self) {
        if let Storage::Mem(bytes) = self {
            // Dropped during GC finalization, which runs on a Ruby thread.
            if let Ok(ruby) = Ruby::get() {
                ruby.gc_adjust_memory_usage(-heap_size(bytes));
            }
        }
    }
}

fn heap_size(bytes: &[u8]) -> isize {
    isize::try_from(bytes.len()).unwrap_or(isize::MAX)
}

// Opens an FST over `storage`. Heap buffers are checksummed first, because the
// fst crate panics on corrupt data and a panic here terminates the Ruby
// process. FSTs written before fst 0.4 carry no checksum and are accepted
// unverified, as they always were. Memory maps skip the check so that opening
// stays lazy; the README documents that they must only be used on trusted files.
fn open_fst(ruby: &Ruby, storage: Storage) -> Result<Fst<Storage>, Error> {
    let verify = matches!(storage, Storage::Mem(_));
    let fst = Fst::new(storage).map_err(|e| fst_error(ruby, e))?;
    if verify {
        match fst.verify() {
            Ok(()) | Err(fst::Error::Fst(fst::raw::Error::ChecksumMissing)) => {}
            Err(e) => return Err(fst_error(ruby, e)),
        }
    }
    Ok(fst)
}

// Runs `f` over the bytes of a Ruby string without copying them. `f` must not
// call into Ruby; pass it pure-Rust work and build Ruby objects from its result.
fn with_bytes<R>(s: RString, f: impl FnOnce(&[u8]) -> R + Send) -> R {
    // SAFETY: the slice borrows memory owned by Ruby, which is sound only while
    // no Ruby code runs, since a yield could mutate the string. The `Send` bound
    // stops `f` from capturing a `Ruby` handle or any Ruby value (all `!Send`),
    // and the callers in this file don't fetch one with `Ruby::get` either, so
    // nothing in `f` can run Ruby code. The higher-ranked bound stops the result
    // borrowing the slice, so it can't outlive this call; `s` is a method
    // argument, so Ruby keeps it alive for the duration.
    f(unsafe { s.as_slice() })
}

// Writes `bytes` to a temporary file beside the target and renames it into
// place, so a reader never sees a partial file and a live memory map of the old
// file is never truncated underneath it. A symlink is resolved first so its
// target is replaced rather than the link, and an existing file's permissions
// carry over. Where a rename can't work — the target isn't a regular file, or
// its directory isn't writable — this writes in place, as `fs::write` would.
fn write_atomically(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let target = match fs::canonicalize(path) {
        Ok(target) => target,
        Err(e) if e.kind() == io::ErrorKind::NotFound && !path.is_symlink() => path.to_path_buf(),
        Err(_) => return fs::write(path, bytes),
    };
    let existing = fs::metadata(&target).ok();
    if existing.as_ref().is_some_and(|meta| !meta.is_file()) {
        return fs::write(&target, bytes);
    }

    let mut tmp_name = OsString::from(target.as_os_str());
    tmp_name.push(format!(".tmp-{}", std::process::id()));
    let tmp_path = PathBuf::from(tmp_name);
    let mut file = match File::create(&tmp_path) {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            return fs::write(&target, bytes);
        }
        Err(e) => return Err(e),
    };

    let result = file
        .write_all(bytes)
        .and_then(|()| match &existing {
            Some(meta) => file.set_permissions(meta.permissions()),
            None => Ok(()),
        })
        .and_then(|()| file.sync_all())
        .and_then(|()| fs::rename(&tmp_path, &target));
    if result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    result
}

// Returns the exclusive upper bound for a prefix scan: the smallest byte
// string strictly greater than `prefix` that does NOT have `prefix` as a
// prefix. None when no such bound exists (empty prefix, or all-0xFF prefix);
// the scan is then unbounded above.
fn prefix_upper_bound(prefix: &[u8]) -> Option<Vec<u8>> {
    let mut upper = prefix.to_vec();
    while let Some(byte) = upper.last_mut() {
        if *byte < 0xFF {
            *byte += 1;
            return Some(upper);
        }
        upper.pop();
    }
    None
}

// ---------------------------------------------------------------------------
// Map and Set share one implementation over the raw FST; they differ only in
// what they yield for each entry. The methods below are generic over that
// difference and registered once per class in `init`.
// ---------------------------------------------------------------------------

#[derive(TypedData)]
#[magnus(class = "RubyFst::Map", free_immediately, size)]
struct FstMap {
    fst: Fst<Storage>,
}

#[derive(TypedData)]
#[magnus(class = "RubyFst::Set", free_immediately, size)]
struct FstSet {
    fst: Fst<Storage>,
}

trait FstObject: TypedData + Sized {
    fn from_fst(fst: Fst<Storage>) -> Self;
    fn fst(&self) -> &Fst<Storage>;
    fn yield_entry(ruby: &Ruby, key: &[u8], value: u64) -> Result<(), Error>;
}

impl FstObject for FstMap {
    fn from_fst(fst: Fst<Storage>) -> Self {
        Self { fst }
    }

    fn fst(&self) -> &Fst<Storage> {
        &self.fst
    }

    fn yield_entry(ruby: &Ruby, key: &[u8], value: u64) -> Result<(), Error> {
        ruby.yield_values::<_, Value>((ruby.str_from_slice(key), value))
            .map(drop)
    }
}

impl FstObject for FstSet {
    fn from_fst(fst: Fst<Storage>) -> Self {
        Self { fst }
    }

    fn fst(&self) -> &Fst<Storage> {
        &self.fst
    }

    fn yield_entry(ruby: &Ruby, key: &[u8], _value: u64) -> Result<(), Error> {
        ruby.yield_value::<_, Value>(ruby.str_from_slice(key))
            .map(drop)
    }
}

impl DataTypeFunctions for FstMap {
    fn size(&self) -> usize {
        size_of_val(self) + heap_bytes(&self.fst)
    }
}

impl DataTypeFunctions for FstSet {
    fn size(&self) -> usize {
        size_of_val(self) + heap_bytes(&self.fst)
    }
}

fn heap_bytes(fst: &Fst<Storage>) -> usize {
    match fst.as_inner() {
        Storage::Mem(bytes) => bytes.len(),
        Storage::Mmap(_) => 0,
    }
}

fn new<T: FstObject>(ruby: &Ruby, bytes: RString) -> Result<T, Error> {
    let bytes = with_bytes(bytes, <[u8]>::to_vec);
    open_fst(ruby, Storage::mem(ruby, bytes)).map(T::from_fst)
}

fn from_path<T: FstObject>(ruby: &Ruby, path: PathBuf) -> Result<T, Error> {
    open_fst(ruby, Storage::read(ruby, &path)?).map(T::from_fst)
}

fn from_path_mmap<T: FstObject>(ruby: &Ruby, path: PathBuf) -> Result<T, Error> {
    open_fst(ruby, Storage::mmap(ruby, &path)?).map(T::from_fst)
}

fn contains<T: FstObject>(rb_self: &T, key: RString) -> bool {
    let fst = rb_self.fst();
    with_bytes(key, |key| fst.contains_key(key))
}

fn len<T: FstObject>(rb_self: &T) -> usize {
    rb_self.fst().len()
}

fn is_empty<T: FstObject>(rb_self: &T) -> bool {
    rb_self.fst().is_empty()
}

fn to_bytes<T: FstObject>(ruby: &Ruby, rb_self: &T) -> RString {
    ruby.str_from_slice(rb_self.fst().as_bytes())
}

fn save<T: FstObject>(ruby: &Ruby, rb_self: &T, path: PathBuf) -> Result<(), Error> {
    write_atomically(&path, rb_self.fst().as_bytes()).map_err(|e| io_error(ruby, &path, e))
}

fn yield_all<T: FstObject, A: Automaton>(
    ruby: &Ruby,
    mut stream: Stream<'_, A>,
) -> Result<(), Error> {
    while let Some((key, output)) = stream.next() {
        T::yield_entry(ruby, key, output.value())?;
    }
    Ok(())
}

fn each<T: FstObject>(ruby: &Ruby, rb_self: &T) -> Result<(), Error> {
    yield_all::<T, _>(ruby, rb_self.fst().stream())
}

fn range<T: FstObject>(
    ruby: &Ruby,
    rb_self: &T,
    ge: Option<RString>,
    le: Option<RString>,
) -> Result<(), Error> {
    let mut builder = rb_self.fst().range();
    if let Some(ge) = ge {
        builder = with_bytes(ge, |bound| builder.ge(bound));
    }
    if let Some(le) = le {
        builder = with_bytes(le, |bound| builder.le(bound));
    }
    yield_all::<T, _>(ruby, builder.into_stream())
}

fn starts_with<T: FstObject>(ruby: &Ruby, rb_self: &T, prefix: RString) -> Result<(), Error> {
    let fst = rb_self.fst();
    let builder = with_bytes(prefix, |prefix| {
        let builder = fst.range().ge(prefix);
        match prefix_upper_bound(prefix) {
            Some(upper) => builder.lt(upper),
            None => builder,
        }
    });
    yield_all::<T, _>(ruby, builder.into_stream())
}

fn search_levenshtein<T: FstObject>(
    ruby: &Ruby,
    rb_self: &T,
    query: String,
    distance: u32,
) -> Result<(), Error> {
    let automaton = Levenshtein::new(&query, distance)
        .map_err(|e| Error::new(ruby.exception_arg_error(), e.to_string()))?;
    yield_all::<T, _>(ruby, rb_self.fst().search(automaton).into_stream())
}

// Map-only lookups. Keys come back as freshly allocated Ruby strings, so each
// lookup finishes with the borrowed key before building its result.

fn get(rb_self: &FstMap, key: RString) -> Option<u64> {
    with_bytes(key, |key| rb_self.fst.get(key).map(Output::value))
}

fn get_le(ruby: &Ruby, rb_self: &FstMap, key: RString) -> Option<(RString, u64)> {
    let (found, value) = with_bytes(key, |key| floor_entry(&rb_self.fst, key))?;
    Some((ruby.str_from_slice(&found), value))
}

fn get_le_value(rb_self: &FstMap, key: RString) -> Option<u64> {
    with_bytes(key, |key| floor_value(&rb_self.fst, key))
}

fn get_ge(ruby: &Ruby, rb_self: &FstMap, key: RString) -> Option<(RString, u64)> {
    let mut stream = with_bytes(key, |key| rb_self.fst.range().ge(key)).into_stream();
    let (found, output) = stream.next()?;
    Some((ruby.str_from_slice(found), output.value()))
}

fn get_ge_value(rb_self: &FstMap, key: RString) -> Option<u64> {
    let builder = with_bytes(key, |key| rb_self.fst.range().ge(key));
    builder
        .into_stream()
        .next()
        .map(|(_, output)| output.value())
}

// ---------------------------------------------------------------------------
// Builders
// ---------------------------------------------------------------------------

#[magnus::wrap(class = "RubyFst::MapBuilder", free_immediately, size)]
struct FstMapBuilder {
    inner: RefCell<Option<fst::MapBuilder<Vec<u8>>>>,
}

impl FstMapBuilder {
    fn new() -> Self {
        Self {
            inner: RefCell::new(Some(fst::MapBuilder::memory())),
        }
    }

    fn insert(ruby: &Ruby, rb_self: &Self, key: RString, value: u64) -> Result<(), Error> {
        let mut guard = rb_self.inner.borrow_mut();
        let builder = guard
            .as_mut()
            .ok_or_else(|| fst_error(ruby, "builder already finished"))?;
        with_bytes(key, |key| builder.insert(key, value)).map_err(|e| map_fst_error(ruby, e))
    }

    fn finish(ruby: &Ruby, rb_self: &Self) -> Result<RString, Error> {
        let builder = rb_self
            .inner
            .borrow_mut()
            .take()
            .ok_or_else(|| fst_error(ruby, "builder already finished"))?;
        let bytes = builder.into_inner().map_err(|e| map_fst_error(ruby, e))?;
        Ok(ruby.str_from_slice(&bytes))
    }
}

#[magnus::wrap(class = "RubyFst::SetBuilder", free_immediately, size)]
struct FstSetBuilder {
    inner: RefCell<Option<fst::SetBuilder<Vec<u8>>>>,
}

impl FstSetBuilder {
    fn new() -> Self {
        Self {
            inner: RefCell::new(Some(fst::SetBuilder::memory())),
        }
    }

    fn insert(ruby: &Ruby, rb_self: &Self, key: RString) -> Result<(), Error> {
        let mut guard = rb_self.inner.borrow_mut();
        let builder = guard
            .as_mut()
            .ok_or_else(|| fst_error(ruby, "builder already finished"))?;
        with_bytes(key, |key| builder.insert(key)).map_err(|e| map_fst_error(ruby, e))
    }

    fn finish(ruby: &Ruby, rb_self: &Self) -> Result<RString, Error> {
        let builder = rb_self
            .inner
            .borrow_mut()
            .take()
            .ok_or_else(|| fst_error(ruby, "builder already finished"))?;
        let bytes = builder.into_inner().map_err(|e| map_fst_error(ruby, e))?;
        Ok(ruby.str_from_slice(&bytes))
    }
}

// ---------------------------------------------------------------------------
// Floor lookup (get_le): greatest key <= query.
//
// The fst crate exposes `range().ge(...)` natively but not a floor operation,
// so this walks the FST manually. While descending along the query it keeps
// the best candidate seen so far: the rightmost transition strictly below the
// query byte at that depth, or else the prefix itself if it is a key. A deeper
// candidate always beats a shallower one, because it agrees with the query for
// longer, so only the latest needs keeping. An exact match beats them all.
// ---------------------------------------------------------------------------

enum Floor {
    // `key[..len]` is itself in the FST, with this output.
    Prefix {
        len: usize,
        output: Output,
    },
    // `key[..len]`, then `input`, then the rightmost path below `addr`.
    Branch {
        len: usize,
        input: u8,
        addr: CompiledAddr,
        output: Output,
    },
}

fn find_floor<D: AsRef<[u8]>>(fst: &Fst<D>, key: &[u8]) -> Option<Floor> {
    let mut node = fst.root();
    let mut output = Output::zero();
    let mut best = None;

    for (depth, &byte) in key.iter().enumerate() {
        if let Some(i) = max_lesser_transition(&node, byte) {
            let t = node.transition(i);
            best = Some(Floor::Branch {
                len: depth,
                input: t.inp,
                addr: t.addr,
                output: output.cat(t.out),
            });
        } else if node.is_final() {
            best = Some(Floor::Prefix {
                len: depth,
                output: output.cat(node.final_output()),
            });
        }

        let Some(i) = node.find_input(byte) else {
            return best;
        };
        let t = node.transition(i);
        output = output.cat(t.out);
        node = fst.node(t.addr);
    }

    if node.is_final() {
        return Some(Floor::Prefix {
            len: key.len(),
            output: output.cat(node.final_output()),
        });
    }
    best
}

fn max_lesser_transition(node: &Node, byte: u8) -> Option<usize> {
    let (mut lo, mut hi) = (0, node.len());
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if node.transition(mid).inp < byte {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo.checked_sub(1)
}

// Follows the last transition from `addr` down to a leaf, passing each input
// byte to `on_byte`, and returns the accumulated value.
fn rightmost<D: AsRef<[u8]>>(
    fst: &Fst<D>,
    addr: CompiledAddr,
    mut output: Output,
    mut on_byte: impl FnMut(u8),
) -> u64 {
    let mut node = fst.node(addr);
    while !node.is_empty() {
        let t = node.transition(node.len() - 1);
        on_byte(t.inp);
        output = output.cat(t.out);
        node = fst.node(t.addr);
    }
    output.cat(node.final_output()).value()
}

fn floor_entry<D: AsRef<[u8]>>(fst: &Fst<D>, key: &[u8]) -> Option<(Vec<u8>, u64)> {
    Some(match find_floor(fst, key)? {
        Floor::Prefix { len, output } => (key[..len].to_vec(), output.value()),
        Floor::Branch {
            len,
            input,
            addr,
            output,
        } => {
            let mut found = key[..len].to_vec();
            found.push(input);
            let value = rightmost(fst, addr, output, |byte| found.push(byte));
            (found, value)
        }
    })
}

fn floor_value<D: AsRef<[u8]>>(fst: &Fst<D>, key: &[u8]) -> Option<u64> {
    Some(match find_floor(fst, key)? {
        Floor::Prefix { output, .. } => output.value(),
        Floor::Branch { addr, output, .. } => rightmost(fst, addr, output, |_| {}),
    })
}

// ---------------------------------------------------------------------------
// Init
// ---------------------------------------------------------------------------

// `method!` can't name a generic parameter from an enclosing function, so the
// shared methods are registered through a macro instead.
macro_rules! define_fst_methods {
    ($class:expr, $type:ty) => {{
        let class = $class;
        class.define_singleton_method("new", function!(new::<$type>, 1))?;
        class.define_singleton_method("from_path", function!(from_path::<$type>, 1))?;
        class.define_singleton_method("from_path_mmap", function!(from_path_mmap::<$type>, 1))?;
        class.define_method("contains?", method!(contains::<$type>, 1))?;
        class.define_method("length", method!(len::<$type>, 0))?;
        class.define_method("size", method!(len::<$type>, 0))?;
        class.define_method("empty?", method!(is_empty::<$type>, 0))?;
        class.define_method("to_bytes", method!(to_bytes::<$type>, 0))?;
        class.define_method("save", method!(save::<$type>, 1))?;
        class.define_method("each", method!(each::<$type>, 0))?;
        class.define_method("range", method!(range::<$type>, 2))?;
        class.define_method("starts_with", method!(starts_with::<$type>, 1))?;
        class.define_method(
            "search_levenshtein",
            method!(search_levenshtein::<$type>, 2),
        )?;
        class
    }};
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("RubyFst")?;
    module.define_error("Error", ruby.exception_runtime_error())?;

    let map_class = define_fst_methods!(module.define_class("Map", ruby.class_object())?, FstMap);
    map_class.define_method("get", method!(get, 1))?;
    map_class.define_method("[]", method!(get, 1))?;
    map_class.define_method("get_le", method!(get_le, 1))?;
    map_class.define_method("get_le_value", method!(get_le_value, 1))?;
    map_class.define_method("get_ge", method!(get_ge, 1))?;
    map_class.define_method("get_ge_value", method!(get_ge_value, 1))?;

    define_fst_methods!(module.define_class("Set", ruby.class_object())?, FstSet);

    let map_builder = module.define_class("MapBuilder", ruby.class_object())?;
    map_builder.define_singleton_method("new", function!(FstMapBuilder::new, 0))?;
    map_builder.define_method("insert", method!(FstMapBuilder::insert, 2))?;
    map_builder.define_method("finish", method!(FstMapBuilder::finish, 0))?;

    let set_builder = module.define_class("SetBuilder", ruby.class_object())?;
    set_builder.define_singleton_method("new", function!(FstSetBuilder::new, 0))?;
    set_builder.define_method("insert", method!(FstSetBuilder::insert, 1))?;
    set_builder.define_method("finish", method!(FstSetBuilder::finish, 0))?;

    Ok(())
}
