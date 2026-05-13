# frozen_string_literal: true

require 'test_helper'

class FloorCeilTest < Minitest::Test
  def build_map(entries)
    b = RubyFst::MapBuilder.new
    entries.sort_by(&:first).each { |k, v| b.insert(k, v) }
    RubyFst::Map.new(b.finish)
  end

  def test_get_le_exact_match
    map = build_map([['bar', 2], ['foo', 1], ['qux', 3]])

    key, val = map.get_le('foo')
    assert_equal('foo', key)
    assert_equal(1, val)
  end

  def test_get_le_between_keys
    map = build_map([['bar', 2], ['foo', 1], ['qux', 3]])

    key, val = map.get_le('dog')
    assert_equal('bar', key)
    assert_equal(2, val)
  end

  def test_get_le_before_all
    map = build_map([['bar', 2], ['foo', 1]])

    assert_nil(map.get_le('aaa'))
  end

  def test_get_le_after_all
    map = build_map([['bar', 2], ['foo', 1]])

    key, val = map.get_le('zzz')
    assert_equal('foo', key)
    assert_equal(1, val)
  end

  def test_get_le_empty_map
    map = build_map([])
    assert_nil(map.get_le('anything'))
  end

  def test_get_le_prefix_key
    map = build_map([['ab', 1], ['abcdef', 2]])

    key, val = map.get_le('abcd')
    assert_equal('ab', key)
    assert_equal(1, val)
  end

  def test_get_le_single_byte_keys
    entries = (0..255).step(16).map { |i| [[i].pack('C'), i.to_i] }
    map = build_map(entries)

    key, val = map.get_le([100].pack('C'))
    assert_equal([96].pack('C'), key)
    assert_equal(96, val)

    key, val = map.get_le([96].pack('C'))
    assert_equal([96].pack('C'), key)
    assert_equal(96, val)

    key, val = map.get_le([0].pack('C'))
    assert_equal([0].pack('C'), key)
    assert_equal(0, val)
  end

  def test_get_le_multibyte_backtrack
    map = build_map([
      ["\x01\xFF".b, 1],
      ["\x02\x00".b, 2],
      ["\x02\x80".b, 3],
    ])

    key, val = map.get_le("\x02\x40".b)
    assert_equal("\x02\x00".b, key)
    assert_equal(2, val)

    key, val = map.get_le("\x01\xFF\xFF".b)
    assert_equal("\x01\xFF".b, key)
    assert_equal(1, val)
  end

  def test_get_le_ip_range_lookup
    ranges = [
      [[167_772_160].pack('N'), 1],   # 10.0.0.0
      [[3_232_235_520].pack('N'), 2], # 192.168.0.0
    ]
    map = build_map(ranges)

    key, val = map.get_le([167_772_260].pack('N'))
    assert_equal([167_772_160].pack('N'), key)
    assert_equal(1, val)

    key, val = map.get_le([3_232_235_570].pack('N'))
    assert_equal([3_232_235_520].pack('N'), key)
    assert_equal(2, val)

    assert_nil(map.get_le([167_772_159].pack('N')))
  end

  def test_get_ge_exact_match
    map = build_map([['bar', 2], ['foo', 1], ['qux', 3]])

    key, val = map.get_ge('foo')
    assert_equal('foo', key)
    assert_equal(1, val)
  end

  def test_get_ge_between_keys
    map = build_map([['bar', 2], ['foo', 1], ['qux', 3]])

    key, val = map.get_ge('dog')
    assert_equal('foo', key)
    assert_equal(1, val)
  end

  def test_get_ge_after_all
    map = build_map([['bar', 2], ['foo', 1]])

    assert_nil(map.get_ge('zzz'))
  end

  def test_get_ge_before_all
    map = build_map([['bar', 2], ['foo', 1]])

    key, val = map.get_ge('aaa')
    assert_equal('bar', key)
    assert_equal(2, val)
  end
end
