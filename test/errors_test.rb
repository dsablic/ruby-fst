# frozen_string_literal: true

require 'test_helper'

class ErrorsTest < Minitest::Test
  def map_bytes
    b = RubyFst::MapBuilder.new
    %w(alpha beta gamma).each_with_index { |w, i| b.insert(w, i) }
    b.finish
  end

  def test_error_is_a_runtime_error
    assert_operator(RubyFst::Error, :<, RuntimeError)
  end

  def test_out_of_order_insert_raises_fst_error
    b = RubyFst::MapBuilder.new
    b.insert('b', 1)
    assert_raises(RubyFst::Error) { b.insert('a', 2) }
  end

  def test_finished_builder_raises_fst_error
    b = RubyFst::SetBuilder.new
    b.finish
    assert_raises(RubyFst::Error) { b.finish }
  end

  def test_garbage_bytes_raise_fst_error
    assert_raises(RubyFst::Error) { RubyFst::Map.new('not an fst') }
  end

  def test_corrupted_bytes_raise_instead_of_crashing
    bytes = map_bytes.dup
    bytes.setbyte(10, bytes.getbyte(10) ^ 0xFF)
    assert_raises(RubyFst::Error) { RubyFst::Map.new(bytes) }
  end

  def test_corrupted_node_data_raises_instead_of_crashing
    bytes = map_bytes.dup
    middle = bytes.bytesize / 2
    bytes.setbyte(middle, bytes.getbyte(middle) ^ 0x5A)
    assert_raises(RubyFst::Error) { RubyFst::Map.new(bytes) }
  end

  def test_set_iterators_without_a_block_return_enumerators
    b = RubyFst::SetBuilder.new
    %w(alpha beta).each { |w| b.insert(w) }
    set = RubyFst::Set.new(b.finish)

    assert_equal(%w(alpha beta), set.each.to_a)
    assert_equal(['beta'], set.search_levenshtein('betx', 1).to_a)
  end

  def test_break_out_of_each_returns_the_break_value
    map = RubyFst::Map.new(map_bytes)

    assert_equal('beta', map.each { |k, _| break k if k.start_with?('b') })
  end

  def test_missing_file_raises_io_error
    assert_raises(IOError) { RubyFst::Set.from_path('/nonexistent/path/xxx.fst') }
  end

  def test_iterators_without_a_block_return_enumerators
    map = RubyFst::Map.new(map_bytes)

    assert_equal([['alpha', 0], ['beta', 1], ['gamma', 2]], map.each.to_a)
    assert_equal(3, map.each.size)
    assert_equal([['beta', 1]], map.search_levenshtein('betx', 1).to_a)
  end

  def test_unbuildable_levenshtein_query_raises_argument_error
    map = RubyFst::Map.new(map_bytes)
    assert_raises(ArgumentError) { map.search_levenshtein('a' * 200, 50) { nil } }
  end
end
