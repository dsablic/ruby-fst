# frozen_string_literal: true

require 'test_helper'

class BuilderTest < Minitest::Test
  def test_map_builder_dropped_without_finish_does_not_crash
    100.times do
      b = RubyFst::MapBuilder.new
      b.insert('a', 1)
      b.insert('b', 2)
    end
    GC.start
    pass
  end

  def test_set_builder_dropped_without_finish_does_not_crash
    100.times do
      b = RubyFst::SetBuilder.new
      b.insert('a')
      b.insert('b')
    end
    GC.start
    pass
  end

  def test_map_builder_insert_after_finish_raises
    b = RubyFst::MapBuilder.new
    b.insert('a', 1)
    b.finish
    assert_raises(RuntimeError) { b.insert('b', 2) }
  end

  def test_set_builder_insert_after_finish_raises
    b = RubyFst::SetBuilder.new
    b.insert('a')
    b.finish
    assert_raises(RuntimeError) { b.insert('b') }
  end

  def test_set_builder_finish_once
    b = RubyFst::SetBuilder.new
    b.insert('a')
    b.finish
    assert_raises(RuntimeError) { b.finish }
  end

  def test_map_builder_rejects_duplicates
    b = RubyFst::MapBuilder.new
    b.insert('a', 1)
    assert_raises(RuntimeError) { b.insert('a', 2) }
  end
end
