# frozen_string_literal: true

require 'test_helper'

class EncodingTest < Minitest::Test
  def build_map
    b = RubyFst::MapBuilder.new
    %w(alpha beta gamma).each_with_index { |w, i| b.insert(w, i) }
    RubyFst::Map.new(b.finish)
  end

  def build_set
    b = RubyFst::SetBuilder.new
    %w(alpha beta gamma).each { |w| b.insert(w) }
    RubyFst::Set.new(b.finish)
  end

  def test_to_bytes_is_binary
    assert_equal(Encoding::BINARY, build_map.to_bytes.encoding)
    assert_equal(Encoding::BINARY, build_set.to_bytes.encoding)
  end

  def test_each_yields_binary_keys
    keys = []
    build_map.each { |k, _| keys << k }
    assert_equal(%w(alpha beta gamma), keys)
    keys.each { |k| assert_equal(Encoding::BINARY, k.encoding) }

    set_keys = []
    build_set.each { |k| set_keys << k }
    set_keys.each { |k| assert_equal(Encoding::BINARY, k.encoding) }
  end

  def test_get_le_returns_binary_key
    key, = build_map.get_le('beta')
    assert_equal(Encoding::BINARY, key.encoding)
  end

  def test_get_ge_returns_binary_key
    key, = build_map.get_ge('beta')
    assert_equal(Encoding::BINARY, key.encoding)
  end

  def test_levenshtein_yields_binary_keys
    keys = []
    build_map.search_levenshtein('alpha', 1) { |k, _| keys << k }
    keys.each { |k| assert_equal(Encoding::BINARY, k.encoding) }
  end

  def test_keys_survive_after_iteration
    map = build_map
    captured = []
    map.each { |k, v| captured << [k, v] }
    GC.start
    assert_equal([['alpha', 0], ['beta', 1], ['gamma', 2]], captured)
  end

  def test_arbitrary_bytes_roundtrip
    b = RubyFst::MapBuilder.new
    b.insert("\x00\x01\xFF".b, 1)
    b.insert("\x01\x00".b, 2)
    map = RubyFst::Map.new(b.finish)

    assert_equal(1, map["\x00\x01\xFF".b])
    assert_equal(2, map["\x01\x00".b])
  end
end
