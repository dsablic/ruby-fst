# frozen_string_literal: true

require_relative 'lib/ruby_fst/version'

Gem::Specification.new do |spec|
  spec.name = 'ruby-fst'
  spec.version = RubyFst::VERSION
  spec.authors = ['Denis Sablic']
  spec.email = ['denis.sablic@gmail.com']
  spec.summary = 'Ruby bindings for the Rust fst crate'
  spec.description = 'Finite state transducer backed ordered sets and maps via the Rust fst crate by BurntSushi'
  spec.homepage = 'https://github.com/dsablic/ruby-fst'
  spec.license = 'MIT'
  spec.required_ruby_version = '>= 3.2'

  spec.metadata['rubygems_mfa_required'] = 'true'
  spec.metadata['source_code_uri'] = 'https://github.com/dsablic/ruby-fst'
  spec.metadata['changelog_uri'] = 'https://github.com/dsablic/ruby-fst/blob/main/CHANGELOG.md'

  spec.files = Dir.chdir(__dir__) do
    `git ls-files -z`.split("\x0").reject { |f| f.start_with?('test/', '.git') }
  end
  spec.extensions = ['ext/ruby_fst/extconf.rb']
  spec.require_paths = ['lib']

  spec.add_dependency('rb_sys', '~> 0.9')
end
