# ruby-fst

[![CI](https://github.com/dsablic/ruby-fst/actions/workflows/ci.yml/badge.svg)](https://github.com/dsablic/ruby-fst/actions/workflows/ci.yml) [![Gem Version](https://badge.fury.io/rb/ruby-fst.svg)](https://badge.fury.io/rb/ruby-fst) [![License](https://img.shields.io/badge/license-MIT-blue.svg)](/LICENSE.txt) [![Ruby](https://img.shields.io/badge/ruby-%3E%3D%203.2-red.svg)](https://www.ruby-lang.org/)

Ruby bindings for the [fst](https://github.com/BurntSushi/fst) crate by Andrew Gallant. Provides finite state transducer backed ordered maps and sets with fast lookup, prefix and range scans, floor/ceiling lookups, and Levenshtein fuzzy search.

## Requirements

- Ruby >= 3.2
- For source installs: a Rust toolchain. Precompiled gems are published for Linux (x86_64, aarch64, glibc and musl), macOS (x86_64, arm64), and Windows (x64) — these install with no Rust toolchain required.

## Installation

```
gem install ruby-fst
```

Or add to your Gemfile:

```ruby
gem 'ruby-fst'
```

## Keys are byte strings

All FST keys are arbitrary byte strings (`Encoding::BINARY`). When a key is returned from the gem (e.g. via `each`, `get_le`, `range`) it always carries the `Encoding::BINARY` encoding — be careful when interpolating into UTF-8 strings or `puts`-ing keys that contain non-ASCII bytes.

Keys must be inserted into a builder in **strictly ascending lexicographic byte order**, with no duplicates. Out-of-order or duplicate inserts raise.

## Map

Ordered map from byte string keys to unsigned 64-bit integer values.

```ruby
require 'ruby_fst'

builder = RubyFst::MapBuilder.new
builder.insert('bar', 2)
builder.insert('baz', 3)
builder.insert('foo', 1)
map = RubyFst::Map.new(builder.finish)

map['foo']             # => 1
map.get('missing')     # => nil
map.contains?('bar')   # => true
map.length             # => 3

map.each { |key, value| puts("#{key}: #{value}") }
```

## Set

Ordered set of byte string keys.

```ruby
builder = RubyFst::SetBuilder.new
builder.insert('bar')
builder.insert('baz')
builder.insert('foo')
set = RubyFst::Set.new(builder.finish)

set.contains?('foo')   # => true
set.length             # => 3

set.each { |key| puts(key) }
```

## Range queries

`Map#range` and `Set#range` stream every entry whose key falls in `[ge, le]`. Either bound may be omitted.

```ruby
map.range(ge: 'b', le: 'f') { |key, value| puts("#{key} -> #{value}") }
map.range(ge: 'b').to_a                      # Enumerator without a block
set.range(le: 'baz') { |key| puts(key) }
```

`Map#starts_with` and `Set#starts_with` stream every entry whose key begins with the given prefix.

```ruby
map.starts_with('foo') { |key, value| puts("#{key} -> #{value}") }
set.starts_with('app').to_a
```

## Floor and ceiling lookups

`get_le` returns the greatest key less than or equal to the query. `get_ge` returns the smallest key greater than or equal to the query. Both return `[key, value]` or `nil`.

```ruby
builder = RubyFst::MapBuilder.new
builder.insert('bar', 1)
builder.insert('foo', 2)
builder.insert('qux', 3)
map = RubyFst::Map.new(builder.finish)

map.get_le('dog')   # => ["bar", 1]
map.get_ge('dog')   # => ["foo", 2]
map.get_le('aaa')   # => nil
```

For range lookups where you only need the value, `get_le_value` and `get_ge_value` skip key reconstruction and return only the `Integer` (or `nil`):

```ruby
map.get_le_value('dog')   # => 1
map.get_ge_value('dog')   # => 2
map.get_le_value('aaa')   # => nil
```

This is useful for IP range lookups. Encode range starts as 4-byte big-endian keys and use `get_le_value` to find which range an IP falls into:

```ruby
require 'ipaddr'

builder = RubyFst::MapBuilder.new
builder.insert([167_772_160].pack('N'), 1)   # 10.0.0.0
builder.insert([3_232_235_520].pack('N'), 2) # 192.168.0.0
map = RubyFst::Map.new(builder.finish)

ip = IPAddr.new('10.0.0.100').to_i
label_id = map.get_le_value([ip].pack('N'))
```

## Levenshtein search

Find all keys within a given edit distance. The search runs as an automaton intersection with the FST, visiting only reachable states.

The query must be valid UTF-8 (the Levenshtein automaton is defined over Unicode codepoints), but the keys themselves can be any bytes — the automaton matches against the UTF-8 interpretation of the keys.

```ruby
builder = RubyFst::MapBuilder.new
%w(bar baz cat foo fun).each_with_index { |w, i| builder.insert(w, i) }
map = RubyFst::Map.new(builder.finish)

map.search_levenshtein('far', 1) { |key, value| puts(key) }
# => bar
```

Works on sets too:

```ruby
set.search_levenshtein('university', 2) { |key| puts(key) }
```

## Serialization

```ruby
# Bytes
bytes = map.to_bytes              # binary string
map = RubyFst::Map.new(bytes)

# File: read entire file into memory (good for small/medium FSTs)
map.save('/path/to/file.fst')
map = RubyFst::Map.from_path('/path/to/file.fst')

# File: memory-map the file (good for large FSTs; lookups page in only what they touch)
map = RubyFst::Map.from_path_mmap('/path/to/file.fst')
```

When using `from_path_mmap`, the file must remain unchanged on disk for the lifetime of the resulting `Map` or `Set` — modifying or truncating it causes undefined behavior.

## Development

```
bundle install
bundle exec rake          # compile + test + rubocop
```

Individual tasks: `rake compile`, `rake test`, `rake rubocop`. Bump the version (and CHANGELOG) with `rake bump[patch]` / `rake bump[minor]` / `rake bump[major]`. Tagging `vX.Y.Z` on GitHub triggers cross-compile and publish to RubyGems.

## License

MIT. See [LICENSE.txt](LICENSE.txt) for details.

This gem wraps the [fst](https://github.com/BurntSushi/fst) crate by Andrew Gallant, also MIT licensed.
