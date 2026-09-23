# frozen_string_literal: true

require 'ruby_fst'

spec = Gem.loaded_specs.fetch('ruby-fst')
abort("Expected a precompiled gem, loaded #{spec.full_name}") if spec.platform.to_s == 'ruby'

binary = $LOADED_FEATURES.grep(/ruby_fst\.(so|bundle)\z/).first
abort('The native extension was not loaded') unless binary

abi = RUBY_VERSION[/\A\d+\.\d+/]
abort("Loaded #{binary}, not the #{abi} build") unless binary.include?("/#{abi}/")

builder = RubyFst::MapBuilder.new
builder.insert('a', 1)
builder.insert('b', 2)
abort('Map lookup returned the wrong value') unless RubyFst::Map.new(builder.finish)['b'] == 2

puts("#{spec.full_name} on Ruby #{RUBY_VERSION}: #{binary}")
