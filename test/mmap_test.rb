# frozen_string_literal: true

require 'test_helper'
require 'tmpdir'

class MmapTest < Minitest::Test
  def with_saved_map
    b = RubyFst::MapBuilder.new
    %w(alpha beta gamma).each_with_index { |w, i| b.insert(w, i) }
    map = RubyFst::Map.new(b.finish)
    path = File.join(Dir.tmpdir, "ruby_fst_mmap_#{Process.pid}_#{rand(1 << 32)}.fst")
    map.save(path)
    yield path
  ensure
    File.delete(path) if path && File.exist?(path)
  end

  def with_saved_set
    b = RubyFst::SetBuilder.new
    %w(alpha beta gamma).each { |w| b.insert(w) }
    set = RubyFst::Set.new(b.finish)
    path = File.join(Dir.tmpdir, "ruby_fst_mmap_set_#{Process.pid}_#{rand(1 << 32)}.fst")
    set.save(path)
    yield path
  ensure
    File.delete(path) if path && File.exist?(path)
  end

  def test_map_from_path_mmap
    with_saved_map do |path|
      map = RubyFst::Map.from_path_mmap(path)

      assert_equal(0, map['alpha'])
      assert_equal(1, map['beta'])
      assert_equal(2, map['gamma'])
      assert_equal(3, map.length)
    end
  end

  def test_map_from_path_mmap_supports_range
    with_saved_map do |path|
      map = RubyFst::Map.from_path_mmap(path)
      keys = []
      map.starts_with('a') { |k, _| keys << k }

      assert_equal(['alpha'], keys)
    end
  end

  def test_set_from_path_mmap
    with_saved_set do |path|
      set = RubyFst::Set.from_path_mmap(path)

      assert(set.contains?('alpha'))
      assert(set.contains?('gamma'))
      refute(set.contains?('zeta'))
    end
  end

  def test_mmap_missing_file_raises
    assert_raises(RuntimeError) { RubyFst::Map.from_path_mmap('/nonexistent/path/xxx.fst') }
  end
end
