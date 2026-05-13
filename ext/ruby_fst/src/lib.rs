use std::cell::RefCell;
use std::fs;

use fst::automaton::Levenshtein;
use fst::raw::{CompiledAddr, Fst, Node, Output};
use fst::{IntoStreamer, Streamer};
use magnus::prelude::*;
use magnus::{block, exception, function, method, Error, RArray, RString, Ruby, Value};

fn err(msg: impl std::fmt::Display) -> Error {
    Error::new(exception::runtime_error(), msg.to_string())
}

fn ruby() -> Ruby {
    unsafe { Ruby::get_unchecked() }
}

// ---------------------------------------------------------------------------
// Map
// ---------------------------------------------------------------------------

#[magnus::wrap(class = "RubyFst::Map", free_immediately, size)]
struct FstMap {
    inner: fst::Map<Vec<u8>>,
}

impl FstMap {
    fn new(bytes: RString) -> Result<Self, Error> {
        let data = unsafe { bytes.as_slice() }.to_vec();
        let inner = fst::Map::new(data).map_err(err)?;
        Ok(Self { inner })
    }

    fn from_path(path: String) -> Result<Self, Error> {
        let data = fs::read(&path).map_err(err)?;
        let inner = fst::Map::new(data).map_err(err)?;
        Ok(Self { inner })
    }

    fn get(&self, key: RString) -> Option<u64> {
        let key = unsafe { key.as_slice() };
        self.inner.get(key)
    }

    fn contains(&self, key: RString) -> bool {
        let key = unsafe { key.as_slice() };
        self.inner.contains_key(key)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn to_bytes(&self) -> RString {
        ruby().str_from_slice(self.inner.as_fst().as_bytes())
    }

    fn save(&self, path: String) -> Result<(), Error> {
        fs::write(&path, self.inner.as_fst().as_bytes()).map_err(err)
    }

    fn get_le(&self, key: RString) -> Result<Option<RArray>, Error> {
        let r = ruby();
        let key = unsafe { key.as_slice() };
        match floor_lookup(self.inner.as_fst(), key) {
            Some((found_key, value)) => {
                let arr = r.ary_new_capa(2);
                arr.push(r.str_from_slice(&found_key))?;
                arr.push(value)?;
                Ok(Some(arr))
            }
            None => Ok(None),
        }
    }

    fn get_ge(&self, key: RString) -> Result<Option<RArray>, Error> {
        let r = ruby();
        let key = unsafe { key.as_slice() };
        let mut stream = self.inner.range().ge(key).into_stream();
        match stream.next() {
            Some((k, v)) => {
                let arr = r.ary_new_capa(2);
                arr.push(r.str_from_slice(k))?;
                arr.push(v)?;
                Ok(Some(arr))
            }
            None => Ok(None),
        }
    }

    fn each(&self) -> Result<(), Error> {
        let r = ruby();
        let mut stream = (&self.inner).into_stream();
        while let Some((key, value)) = stream.next() {
            let rb_key = r.str_from_slice(key);
            let _: Value = block::yield_values((rb_key, value))?;
        }
        Ok(())
    }

    fn search_levenshtein(&self, query: String, distance: u32) -> Result<(), Error> {
        let r = ruby();
        let lev = Levenshtein::new(&query, distance).map_err(err)?;
        let mut stream = self.inner.search(lev).into_stream();
        while let Some((key, value)) = stream.next() {
            let rb_key = r.str_from_slice(key);
            let _: Value = block::yield_values((rb_key, value))?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// MapBuilder
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

    fn insert(&self, key: RString, value: u64) -> Result<(), Error> {
        let key = unsafe { key.as_slice() }.to_vec();
        let mut guard = self.inner.borrow_mut();
        let b = guard.as_mut().ok_or_else(|| err("builder already finished"))?;
        b.insert(&key, value).map_err(err)
    }

    fn finish(&self) -> Result<RString, Error> {
        let mut guard = self.inner.borrow_mut();
        let b = guard.take().ok_or_else(|| err("builder already finished"))?;
        let bytes = b.into_inner().map_err(err)?;
        Ok(ruby().str_from_slice(&bytes))
    }
}

// ---------------------------------------------------------------------------
// Set
// ---------------------------------------------------------------------------

#[magnus::wrap(class = "RubyFst::Set", free_immediately, size)]
struct FstSet {
    inner: fst::Set<Vec<u8>>,
}

impl FstSet {
    fn new(bytes: RString) -> Result<Self, Error> {
        let data = unsafe { bytes.as_slice() }.to_vec();
        let inner = fst::Set::new(data).map_err(err)?;
        Ok(Self { inner })
    }

    fn from_path(path: String) -> Result<Self, Error> {
        let data = fs::read(&path).map_err(err)?;
        let inner = fst::Set::new(data).map_err(err)?;
        Ok(Self { inner })
    }

    fn contains(&self, key: RString) -> bool {
        let key = unsafe { key.as_slice() };
        self.inner.contains(key)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn to_bytes(&self) -> RString {
        ruby().str_from_slice(self.inner.as_fst().as_bytes())
    }

    fn save(&self, path: String) -> Result<(), Error> {
        fs::write(&path, self.inner.as_fst().as_bytes()).map_err(err)
    }

    fn each(&self) -> Result<(), Error> {
        let r = ruby();
        let mut stream = (&self.inner).into_stream();
        while let Some(key) = stream.next() {
            let rb_key = r.str_from_slice(key);
            let _: Value = block::yield_value(rb_key)?;
        }
        Ok(())
    }

    fn search_levenshtein(&self, query: String, distance: u32) -> Result<(), Error> {
        let r = ruby();
        let lev = Levenshtein::new(&query, distance).map_err(err)?;
        let mut stream = self.inner.search(lev).into_stream();
        while let Some(key) = stream.next() {
            let rb_key = r.str_from_slice(key);
            let _: Value = block::yield_value(rb_key)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// SetBuilder
// ---------------------------------------------------------------------------

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

    fn insert(&self, key: RString) -> Result<(), Error> {
        let key = unsafe { key.as_slice() }.to_vec();
        let mut guard = self.inner.borrow_mut();
        let b = guard.as_mut().ok_or_else(|| err("builder already finished"))?;
        b.insert(&key).map_err(err)
    }

    fn finish(&self) -> Result<RString, Error> {
        let mut guard = self.inner.borrow_mut();
        let b = guard.take().ok_or_else(|| err("builder already finished"))?;
        let bytes = b.into_inner().map_err(err)?;
        Ok(ruby().str_from_slice(&bytes))
    }
}

// ---------------------------------------------------------------------------
// Floor lookup (get_le): greatest key <= query
// ---------------------------------------------------------------------------

struct Frame {
    node_addr: CompiledAddr,
    output: Output,
    prefix_len: usize,
    max_lesser_idx: Option<usize>,
    is_final: bool,
    final_value: u64,
}

fn find_max_lesser(node: &Node, byte: u8) -> Option<usize> {
    let n = node.len();
    if n == 0 {
        return None;
    }
    let mut lo: usize = 0;
    let mut hi: usize = n;
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

fn rightmost_to_leaf<D: AsRef<[u8]>>(
    fst: &Fst<D>,
    addr: CompiledAddr,
    output: Output,
) -> (Vec<u8>, u64) {
    let mut node = fst.node(addr);
    let mut out = output;
    let mut suffix = Vec::new();

    while node.len() > 0 {
        let last = node.len() - 1;
        let t = node.transition(last);
        suffix.push(t.inp);
        out = out.cat(t.out);
        node = fst.node(t.addr);
    }

    (suffix, out.cat(node.final_output()).value())
}

fn floor_lookup<D: AsRef<[u8]>>(fst: &Fst<D>, key: &[u8]) -> Option<(Vec<u8>, u64)> {
    let root = fst.root();

    if key.is_empty() {
        return if root.is_final() {
            Some((Vec::new(), root.final_output().value()))
        } else {
            None
        };
    }

    let mut node = root;
    let mut output = Output::zero();
    let mut stack: Vec<Frame> = Vec::with_capacity(key.len());
    let mut matched: usize = 0;

    for &byte in key.iter() {
        let lesser = find_max_lesser(&node, byte);

        stack.push(Frame {
            node_addr: node.addr(),
            output,
            prefix_len: matched,
            max_lesser_idx: lesser,
            is_final: node.is_final(),
            final_value: output.cat(node.final_output()).value(),
        });

        match node.find_input(byte) {
            Some(idx) => {
                let t = node.transition(idx);
                output = output.cat(t.out);
                node = fst.node(t.addr);
                matched += 1;
            }
            None => break,
        }
    }

    if matched == key.len() && node.is_final() {
        return Some((key.to_vec(), output.cat(node.final_output()).value()));
    }

    while let Some(frame) = stack.pop() {
        if let Some(j) = frame.max_lesser_idx {
            let frame_node = fst.node(frame.node_addr);
            let t = frame_node.transition(j);
            let mut result = key[..frame.prefix_len].to_vec();
            result.push(t.inp);
            let branch_output = frame.output.cat(t.out);
            let (suffix, val) = rightmost_to_leaf(fst, t.addr, branch_output);
            result.extend(suffix);
            return Some((result, val));
        }

        if frame.is_final {
            return Some((key[..frame.prefix_len].to_vec(), frame.final_value));
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Init
// ---------------------------------------------------------------------------

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("RubyFst")?;

    let map_class = module.define_class("Map", ruby.class_object())?;
    map_class.define_singleton_method("new", function!(FstMap::new, 1))?;
    map_class.define_singleton_method("from_path", function!(FstMap::from_path, 1))?;
    map_class.define_method("get", method!(FstMap::get, 1))?;
    map_class.define_method("[]", method!(FstMap::get, 1))?;
    map_class.define_method("contains?", method!(FstMap::contains, 1))?;
    map_class.define_method("length", method!(FstMap::len, 0))?;
    map_class.define_method("size", method!(FstMap::len, 0))?;
    map_class.define_method("empty?", method!(FstMap::is_empty, 0))?;
    map_class.define_method("to_bytes", method!(FstMap::to_bytes, 0))?;
    map_class.define_method("save", method!(FstMap::save, 1))?;
    map_class.define_method("get_le", method!(FstMap::get_le, 1))?;
    map_class.define_method("get_ge", method!(FstMap::get_ge, 1))?;
    map_class.define_method("each", method!(FstMap::each, 0))?;
    map_class.define_method("search_levenshtein", method!(FstMap::search_levenshtein, 2))?;

    let map_builder = module.define_class("MapBuilder", ruby.class_object())?;
    map_builder.define_singleton_method("new", function!(FstMapBuilder::new, 0))?;
    map_builder.define_method("insert", method!(FstMapBuilder::insert, 2))?;
    map_builder.define_method("finish", method!(FstMapBuilder::finish, 0))?;

    let set_class = module.define_class("Set", ruby.class_object())?;
    set_class.define_singleton_method("new", function!(FstSet::new, 1))?;
    set_class.define_singleton_method("from_path", function!(FstSet::from_path, 1))?;
    set_class.define_method("contains?", method!(FstSet::contains, 1))?;
    set_class.define_method("length", method!(FstSet::len, 0))?;
    set_class.define_method("size", method!(FstSet::len, 0))?;
    set_class.define_method("empty?", method!(FstSet::is_empty, 0))?;
    set_class.define_method("to_bytes", method!(FstSet::to_bytes, 0))?;
    set_class.define_method("save", method!(FstSet::save, 1))?;
    set_class.define_method("each", method!(FstSet::each, 0))?;
    set_class.define_method("search_levenshtein", method!(FstSet::search_levenshtein, 2))?;

    let set_builder = module.define_class("SetBuilder", ruby.class_object())?;
    set_builder.define_singleton_method("new", function!(FstSetBuilder::new, 0))?;
    set_builder.define_method("insert", method!(FstSetBuilder::insert, 1))?;
    set_builder.define_method("finish", method!(FstSetBuilder::finish, 0))?;

    Ok(())
}
