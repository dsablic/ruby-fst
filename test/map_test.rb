# frozen_string_literal: true

require 'test_helper'

class MapTest < Minitest::Test
  def build_map(entries)
    b = RubyFst::MapBuilder.new
    entries.sort_by(&:first).each { |k, v| b.insert(k, v) }
    RubyFst::Map.new(b.finish)
  end

  def test_get_and_contains
    map = build_map([['bar', 2], ['baz', 3], ['foo', 1]])

    assert_equal(1, map.get('foo'))
    assert_equal(2, map['bar'])
    assert_equal(3, map['baz'])
    assert_nil(map.get('missing'))
    assert(map.contains?('foo'))
    refute(map.contains?('missing'))
  end

  def test_length_and_empty
    empty = build_map([])

    assert_equal(0, empty.length)
    assert_empty(empty)

    map = build_map([['a', 1], ['b', 2]])

    assert_equal(2, map.size)
    refute_empty(map)
  end

  def test_each_enumerable
    map = build_map([['a', 1], ['b', 2], ['c', 3]])
    pairs = []
    map.each { |k, v| pairs << [k, v] }

    assert_equal([['a', 1], ['b', 2], ['c', 3]], pairs)
    assert_equal(%w(a b c), map.map { |k, _| k })
  end

  def test_roundtrip_bytes
    original = build_map([['x', 10], ['y', 20]])
    bytes = original.to_bytes
    restored = RubyFst::Map.new(bytes)

    assert_equal(10, restored['x'])
    assert_equal(20, restored['y'])
    assert_equal(2, restored.length)
  end

  def test_save_and_from_path
    map = build_map([['hello', 42]])
    path = File.join(Dir.tmpdir, "ruby_fst_test_#{Process.pid}.fst")

    begin
      map.save(path)
      loaded = RubyFst::Map.from_path(path)

      assert_equal(42, loaded['hello'])
    ensure
      FileUtils.rm_f(path)
    end
  end

  def test_builder_rejects_out_of_order
    b = RubyFst::MapBuilder.new
    b.insert('b', 1)
    assert_raises(RuntimeError) { b.insert('a', 2) }
  end

  def test_builder_finish_once
    b = RubyFst::MapBuilder.new
    b.insert('a', 1)
    b.finish
    assert_raises(RuntimeError) { b.finish }
  end
end
