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
