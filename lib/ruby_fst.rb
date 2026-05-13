# frozen_string_literal: true

require_relative 'ruby_fst/version'
require_relative 'ruby_fst/ruby_fst'

module RubyFst
  module RangeQuery
    def range(ge: nil, le: nil, &block)
      return enum_for(:range, ge:, le:) unless block

      _range(ge, le, &block)
    end

    def starts_with(prefix, &block)
      return enum_for(:starts_with, prefix) unless block

      _starts_with(prefix, &block)
    end
  end

  class Map
    include Enumerable

    alias _range range
    alias _starts_with starts_with
    private :_range, :_starts_with

    prepend RangeQuery
  end

  class Set
    include Enumerable

    alias _range range
    alias _starts_with starts_with
    private :_range, :_starts_with

    prepend RangeQuery
  end
end
