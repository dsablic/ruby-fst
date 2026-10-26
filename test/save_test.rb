# frozen_string_literal: true

require 'test_helper'
require 'pathname'
require 'tmpdir'

class SaveTest < Minitest::Test
  def setup
    b = RubyFst::MapBuilder.new
    %w(alpha beta gamma).each_with_index { |w, i| b.insert(w, i) }
    @map = RubyFst::Map.new(b.finish)
  end

  def windows?
    Gem.win_platform?
  end

  def test_save_and_load_accept_pathname
    Dir.mktmpdir do |dir|
      path = Pathname(dir).join('map.fst')
      @map.save(path)

      assert_equal(1, RubyFst::Map.from_path(path)['beta'])
    end
  end

  def test_save_through_symlink_updates_the_target
    skip('symlinks need privileges on Windows') if windows?
    Dir.mktmpdir do |dir|
      target = File.join(dir, 'real.fst')
      link = File.join(dir, 'link.fst')
      File.write(target, 'old')
      File.symlink(target, link)
      @map.save(link)

      assert(File.symlink?(link))
      assert_equal(2, RubyFst::Map.from_path(target)['gamma'])
    end
  end

  def test_save_keeps_existing_permissions
    skip('POSIX permissions') if windows?
    Dir.mktmpdir do |dir|
      path = File.join(dir, 'map.fst')
      File.write(path, 'old')
      File.chmod(0o640, path)
      @map.save(path)

      assert_equal(0o640, File.stat(path).mode & 0o777)
    end
  end

  def test_save_into_read_only_directory_writes_in_place
    skip('POSIX permissions') if windows? || Process.uid.zero?
    Dir.mktmpdir do |dir|
      path = File.join(dir, 'map.fst')
      File.write(path, 'old')
      File.chmod(0o555, dir)
      @map.save(path)

      assert_equal(0, RubyFst::Map.from_path(path)['alpha'])
    ensure
      File.chmod(0o755, dir)
    end
  end

  def test_save_leaves_no_temporary_files
    Dir.mktmpdir do |dir|
      @map.save(File.join(dir, 'map.fst'))

      assert_equal(['map.fst'], Dir.children(dir))
    end
  end
end
