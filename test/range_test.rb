# frozen_string_literal: true

require 'test_helper'

class RangeTest < Minitest::Test
  def build_map
    b = RubyFst::MapBuilder.new
    %w(apple application apply banana cat catalog).each_with_index do |w, i|
      b.insert(w, i)
    end
    RubyFst::Map.new(b.finish)
  end

  def build_set
    b = RubyFst::SetBuilder.new
    %w(apple application apply banana cat catalog).each { |w| b.insert(w) }
    RubyFst::Set.new(b.finish)
  end

  def test_map_range_both_bounds
    pairs = []
    build_map.range(ge: 'apply', le: 'cat') { |k, v| pairs << [k, v] }
    assert_equal([['apply', 2], ['banana', 3], ['cat', 4]], pairs)
  end

  def test_map_range_ge_only
    pairs = []
    build_map.range(ge: 'banana') { |k, v| pairs << [k, v] }
    assert_equal([['banana', 3], ['cat', 4], ['catalog', 5]], pairs)
  end

  def test_map_range_le_only
    pairs = []
    build_map.range(le: 'banana') { |k, v| pairs << [k, v] }
    assert_equal([['apple', 0], ['application', 1], ['apply', 2], ['banana', 3]], pairs)
  end

  def test_map_range_returns_enumerator_without_block
    enum = build_map.range(ge: 'banana')
    assert_kind_of(Enumerator, enum)
    assert_equal([['banana', 3], ['cat', 4], ['catalog', 5]], enum.to_a)
  end

  def test_map_starts_with
    pairs = []
    build_map.starts_with('app') { |k, v| pairs << [k, v] }
    assert_equal([['apple', 0], ['application', 1], ['apply', 2]], pairs)
  end

  def test_map_starts_with_no_matches
    pairs = []
    build_map.starts_with('zzz') { |k, v| pairs << [k, v] }
    assert_empty(pairs)
  end

  def test_map_starts_with_empty_prefix_yields_all
    pairs = []
    build_map.starts_with('') { |k, v| pairs << [k, v] }
    assert_equal(6, pairs.length)
  end

  def test_set_range
    keys = []
    build_set.range(ge: 'apply', le: 'cat') { |k| keys << k }
    assert_equal(%w(apply banana cat), keys)
  end

  def test_set_starts_with
    keys = []
    build_set.starts_with('app') { |k| keys << k }
    assert_equal(%w(apple application apply), keys)
  end

  def test_starts_with_handles_0xff_bytes
    b = RubyFst::MapBuilder.new
    b.insert("\xFE".b, 1)
    b.insert("\xFF".b, 2)
    b.insert("\xFF\x00".b, 3)
    b.insert("\xFF\xFF".b, 4)
    map = RubyFst::Map.new(b.finish)

    pairs = []
    map.starts_with("\xFF".b) { |k, v| pairs << [k, v] }
    assert_equal([["\xFF".b, 2], ["\xFF\x00".b, 3], ["\xFF\xFF".b, 4]], pairs)
  end
end
