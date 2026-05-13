# frozen_string_literal: true

require 'test_helper'

class SetTest < Minitest::Test
  def build_set(keys)
    b = RubyFst::SetBuilder.new
    keys.sort.each { |k| b.insert(k) }
    RubyFst::Set.new(b.finish)
  end

  def test_contains
    set = build_set(%w(bar baz foo))

    assert(set.contains?('foo'))
    assert(set.contains?('bar'))
    refute(set.contains?('missing'))
  end

  def test_length_and_empty
    empty = build_set([])

    assert_equal(0, empty.length)
    assert_empty(empty)

    set = build_set(%w(a b))

    assert_equal(2, set.size)
    refute_empty(set)
  end

  def test_each_enumerable
    set = build_set(%w(a b c))
    keys = []
    set.each { |k| keys << k }

    assert_equal(%w(a b c), keys)
    assert_equal(%w(a b c), set.to_a)
  end

  def test_roundtrip_bytes
    original = build_set(%w(x y z))
    restored = RubyFst::Set.new(original.to_bytes)

    assert(restored.contains?('x'))
    assert(restored.contains?('z'))
    assert_equal(3, restored.length)
  end

  def test_save_and_from_path
    set = build_set(%w(hello))
    path = File.join(Dir.tmpdir, "ruby_fst_set_test_#{Process.pid}.fst")

    begin
      set.save(path)
      loaded = RubyFst::Set.from_path(path)

      assert(loaded.contains?('hello'))
    ensure
      FileUtils.rm_f(path)
    end
  end

  def test_builder_rejects_out_of_order
    b = RubyFst::SetBuilder.new
    b.insert('b')
    assert_raises(RuntimeError) { b.insert('a') }
  end
end
