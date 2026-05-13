# frozen_string_literal: true

require 'rake/testtask'
require 'rb_sys/extensiontask'
require 'rubocop/rake_task'

GEMSPEC = Gem::Specification.load('ruby_fst.gemspec')

RbSys::ExtensionTask.new('ruby_fst', GEMSPEC) do |ext|
  ext.lib_dir = 'lib/ruby_fst'
  ext.cross_compile = true
  ext.cross_platform = %w(
    aarch64-linux
    aarch64-linux-musl
    arm64-darwin
    x64-mingw-ucrt
    x86_64-darwin
    x86_64-linux
    x86_64-linux-musl
  )
end

Rake::TestTask.new do |t|
  t.libs << 'test'
  t.test_files = FileList['test/**/*_test.rb']
end

RuboCop::RakeTask.new

task default: %i(compile test rubocop)

desc 'Bump version (rake bump[patch], rake bump[minor], rake bump[major])'
task :bump, [:level] do |_, args|
  level = args[:level] || 'patch'
  version_file = File.join(__dir__, 'lib', 'ruby_fst', 'version.rb')
  cargo_file = File.join(__dir__, 'ext', 'ruby_fst', 'Cargo.toml')
  lock_file = File.join(__dir__, 'Cargo.lock')

  content = File.read(version_file)
  current = content[/VERSION = '(.+)'/, 1]
  major, minor, patch = current.split('.').map(&:to_i)

  new_version =
    case level
    when 'major' then "#{major + 1}.0.0"
    when 'minor' then "#{major}.#{minor + 1}.0"
    when 'patch' then "#{major}.#{minor}.#{patch + 1}"
    else abort("Unknown level: #{level}. Use major, minor, or patch.")
    end

  File.write(version_file, content.sub(/VERSION = '.+'/, "VERSION = '#{new_version}'"))

  cargo = File.read(cargo_file)
  File.write(cargo_file, cargo.sub(/^version = ".+"/, "version = \"#{new_version}\""))

  if File.exist?(lock_file)
    lock = File.read(lock_file)
    File.write(
      lock_file,
      lock.sub(
        /(\[\[package\]\]\nname = "ruby_fst"\nversion = ").+(")/,
        "\\1#{new_version}\\2"
      )
    )
  end

  changelog_file = File.join(__dir__, 'CHANGELOG.md')
  if File.exist?(changelog_file)
    changelog = File.read(changelog_file)
    today = Time.now.utc.strftime('%Y-%m-%d')
    promoted = changelog.sub(
      '## [Unreleased]',
      "## [Unreleased]\n\n## [#{new_version}] — #{today}"
    )
    File.write(changelog_file, promoted) if promoted != changelog
  end

  puts("#{current} -> #{new_version}")
end
