# frozen_string_literal: true

require 'test_helper'

class LevenshteinTest < Minitest::Test
  def test_map_search_levenshtein
    b = RubyFst::MapBuilder.new
    %w(bar baz cat foo fun).each_with_index { |w, i| b.insert(w, i) }
    map = RubyFst::Map.new(b.finish)

    results = []
    map.search_levenshtein('far', 1) { |k, v| results << [k, v] }

    assert_includes(results, ['bar', 0])
    refute(results.any? { |k, _| k == 'baz' })
  end

  def test_map_search_levenshtein_distance_0
    b = RubyFst::MapBuilder.new
    %w(bar baz foo).each_with_index { |w, i| b.insert(w, i) }
    map = RubyFst::Map.new(b.finish)

    results = []
    map.search_levenshtein('foo', 0) { |k, v| results << [k, v] }

    assert_equal([['foo', 2]], results)
  end

  def test_map_search_levenshtein_distance_2
    b = RubyFst::MapBuilder.new
    %w(bar baz cat foo fun).each_with_index { |w, i| b.insert(w, i) }
    map = RubyFst::Map.new(b.finish)

    results = []
    map.search_levenshtein('bax', 2) { |k, v| results << [k, v] }

    keys = results.map(&:first)
    assert_includes(keys, 'bar')
    assert_includes(keys, 'baz')
    assert_includes(keys, 'cat')
  end

  def test_map_search_levenshtein_no_matches
    b = RubyFst::MapBuilder.new
    %w(aaa bbb ccc).each_with_index { |w, i| b.insert(w, i) }
    map = RubyFst::Map.new(b.finish)

    results = []
    map.search_levenshtein('zzz', 1) { |k, v| results << [k, v] }

    assert_empty(results)
  end

  def test_set_search_levenshtein
    b = RubyFst::SetBuilder.new
    %w(bar baz cat foo fun).each { |w| b.insert(w) }
    set = RubyFst::Set.new(b.finish)

    results = []
    set.search_levenshtein('far', 1) { |k| results << k }

    assert_includes(results, 'bar')
    refute_includes(results, 'baz')
  end

  def test_set_search_levenshtein_no_matches
    b = RubyFst::SetBuilder.new
    %w(aaa bbb ccc).each { |w| b.insert(w) }
    set = RubyFst::Set.new(b.finish)

    results = []
    set.search_levenshtein('zzz', 1) { |k| results << k }

    assert_empty(results)
  end
end
