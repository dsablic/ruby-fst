# frozen_string_literal: true

require 'rake/testtask'
require 'rb_sys/extensiontask'

GEMSPEC = Gem::Specification.load('ruby_fst.gemspec')

RbSys::ExtensionTask.new('ruby_fst', GEMSPEC) do |ext|
  ext.lib_dir = 'lib/ruby_fst'
end

Rake::TestTask.new do |t|
  t.libs << 'test'
  t.test_files = FileList['test/**/*_test.rb']
end

task default: %i(compile test)

desc 'Bump version (rake bump[patch], rake bump[minor], rake bump[major])'
task :bump, [:level] do |_, args|
  level = args[:level] || 'patch'
  version_file = File.join(__dir__, 'lib', 'ruby_fst', 'version.rb')
  cargo_file = File.join(__dir__, 'ext', 'ruby_fst', 'Cargo.toml')

  content = File.read(version_file)
  current = content[/VERSION = '(.+)'/, 1]
  major, minor, patch = current.split('.').map(&:to_i)

  case level
  when 'major' then major += 1; minor = 0; patch = 0
  when 'minor' then minor += 1; patch = 0
  when 'patch' then patch += 1
  else abort("Unknown level: #{level}. Use major, minor, or patch.")
  end

  new_version = "#{major}.#{minor}.#{patch}"

  File.write(version_file, content.sub(/VERSION = '.+'/, "VERSION = '#{new_version}'"))

  cargo = File.read(cargo_file)
  File.write(cargo_file, cargo.sub(/^version = ".+"/, "version = \"#{new_version}\""))

  puts("#{current} -> #{new_version}")
end
