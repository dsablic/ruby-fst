# frozen_string_literal: true

require 'test_helper'

class FloorValueTest < Minitest::Test
  def build_map(entries)
    b = RubyFst::MapBuilder.new
    entries.sort_by(&:first).each { |k, v| b.insert(k, v) }
    RubyFst::Map.new(b.finish)
  end

  def test_get_le_value_exact_match
    map = build_map([['bar', 2], ['foo', 1], ['qux', 3]])

    assert_equal(1, map.get_le_value('foo'))
  end

  def test_get_le_value_between_keys
    map = build_map([['bar', 2], ['foo', 1], ['qux', 3]])

    assert_equal(2, map.get_le_value('dog'))
  end

  def test_get_le_value_before_all
    map = build_map([['bar', 2], ['foo', 1]])

    assert_nil(map.get_le_value('aaa'))
  end

  def test_get_le_value_after_all
    map = build_map([['bar', 2], ['foo', 1]])

    assert_equal(1, map.get_le_value('zzz'))
  end

  def test_get_le_value_empty_map
    assert_nil(build_map([]).get_le_value('anything'))
  end

  def test_get_le_value_prefix_key
    map = build_map([['ab', 1], ['abcdef', 2]])

    assert_equal(1, map.get_le_value('abcd'))
  end

  def test_get_le_value_matches_get_le
    entries = (0..255).step(16).map { |i| [[i].pack('C'), i.to_i] }
    map = build_map(entries)

    [[0].pack('C'), [42].pack('C'), [96].pack('C'), [255].pack('C')].each do |query|
      _, expected = map.get_le(query)
      assert_equal(expected, map.get_le_value(query))
    end
  end

  def test_get_le_value_ip_range_lookup
    ranges = [
      [[167_772_160].pack('N'), 1],
      [[3_232_235_520].pack('N'), 2],
    ]
    map = build_map(ranges)

    assert_equal(1, map.get_le_value([167_772_260].pack('N')))
    assert_equal(2, map.get_le_value([3_232_235_570].pack('N')))
    assert_nil(map.get_le_value([167_772_159].pack('N')))
  end

  def test_get_ge_value_exact_match
    map = build_map([['bar', 2], ['foo', 1], ['qux', 3]])

    assert_equal(1, map.get_ge_value('foo'))
  end

  def test_get_ge_value_between_keys
    map = build_map([['bar', 2], ['foo', 1], ['qux', 3]])

    assert_equal(1, map.get_ge_value('dog'))
  end

  def test_get_ge_value_after_all
    map = build_map([['bar', 2], ['foo', 1]])

    assert_nil(map.get_ge_value('zzz'))
  end

  def test_get_ge_value_before_all
    map = build_map([['bar', 2], ['foo', 1]])

    assert_equal(2, map.get_ge_value('aaa'))
  end
end
