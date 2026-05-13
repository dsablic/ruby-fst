# ruby-fst

[![CI](https://github.com/dsablic/ruby-fst/actions/workflows/ci.yml/badge.svg)](https://github.com/dsablic/ruby-fst/actions/workflows/ci.yml) [![Gem Version](https://badge.fury.io/rb/ruby-fst.svg)](https://badge.fury.io/rb/ruby-fst) [![License](https://img.shields.io/badge/license-MIT-blue.svg)](/LICENSE.txt) [![Ruby](https://img.shields.io/badge/ruby-%3E%3D%203.0-red.svg)](https://www.ruby-lang.org/)

Ruby bindings for the [fst](https://github.com/BurntSushi/fst) crate by Andrew Gallant. Provides finite state transducer backed ordered maps and sets with fast lookup, range queries, and fuzzy search.

## Requirements

- Ruby >= 3.0
- Rust toolchain (for compilation)

## Installation

```ruby
gem 'ruby-fst'
```

## Usage

### Map

Ordered map from byte string keys to unsigned 64-bit integer values. Keys must be inserted in lexicographic order.

```ruby
require 'ruby_fst'

builder = RubyFst::MapBuilder.new
builder.insert('bar', 2)
builder.insert('baz', 3)
builder.insert('foo', 1)
map = RubyFst::Map.new(builder.finish)

map['foo']        # => 1
map.get('missing') # => nil
map.contains?('bar') # => true
map.length        # => 3

map.each { |key, value| puts "#{key}: #{value}" }
```

### Set

Ordered set of byte string keys.

```ruby
builder = RubyFst::SetBuilder.new
builder.insert('bar')
builder.insert('baz')
builder.insert('foo')
set = RubyFst::Set.new(builder.finish)

set.contains?('foo') # => true
set.length           # => 3

set.each { |key| puts key }
```

### Floor and ceiling lookups

`get_le` returns the greatest key less than or equal to the query. `get_ge` returns the smallest key greater than or equal to the query. Both return `[key, value]` or `nil`.

```ruby
builder = RubyFst::MapBuilder.new
builder.insert('bar', 1)
builder.insert('foo', 2)
builder.insert('qux', 3)
map = RubyFst::Map.new(builder.finish)

map.get_le('dog') # => ["bar", 1]
map.get_ge('dog') # => ["foo", 2]
map.get_le('aaa') # => nil
```

This is useful for IP range lookups. Encode range starts as 4-byte big-endian keys and use `get_le` to find which range an IP falls into:

```ruby
builder = RubyFst::MapBuilder.new
builder.insert([167_772_160].pack('N'), 1)  # 10.0.0.0
builder.insert([3_232_235_520].pack('N'), 2) # 192.168.0.0
map = RubyFst::Map.new(builder.finish)

ip = IPAddr.new('10.0.0.100').to_i
key, label_id = map.get_le([ip].pack('N'))
```

### Levenshtein search

Find all keys within a given edit distance. The search runs as an automaton intersection with the FST, visiting only reachable states.

```ruby
builder = RubyFst::MapBuilder.new
%w(bar baz cat foo fun).each_with_index { |w, i| builder.insert(w, i) }
map = RubyFst::Map.new(builder.finish)

map.search_levenshtein('far', 1) { |key, value| puts key }
# => bar
```

Works on sets too:

```ruby
set.search_levenshtein('university', 2) { |key| puts key }
```

### Serialization

```ruby
# To/from bytes
bytes = map.to_bytes
map = RubyFst::Map.new(bytes)

# To/from file
map.save('/path/to/file.fst')
map = RubyFst::Map.from_path('/path/to/file.fst')
```

## Development

```
bundle install
bundle exec rake compile test
```

## License

MIT. See [LICENSE.txt](LICENSE.txt) for details.

This gem wraps the [fst](https://github.com/BurntSushi/fst) crate by Andrew Gallant, also MIT licensed.
