# frozen_string_literal: true

require_relative 'ruby_fst/version'
begin
  require_relative "ruby_fst/#{RUBY_VERSION[/\A\d+\.\d+/]}/ruby_fst"
rescue LoadError
  require_relative 'ruby_fst/ruby_fst'
end

module RubyFst
  module RangeQuery
    def each(&block)
      return enum_for(:each) { size } unless block

      _each(&block)
    end

    def range(ge: nil, le: nil, &block)
      return enum_for(:range, ge:, le:) unless block

      _range(ge, le, &block)
    end

    def starts_with(prefix, &block)
      return enum_for(:starts_with, prefix) unless block

      _starts_with(prefix, &block)
    end

    def search_levenshtein(query, distance, &block)
      return enum_for(:search_levenshtein, query, distance) unless block

      _search_levenshtein(query, distance, &block)
    end
  end

  class Map
    include Enumerable

    alias _each each
    alias _range range
    alias _starts_with starts_with
    alias _search_levenshtein search_levenshtein
    private :_each, :_range, :_starts_with, :_search_levenshtein

    prepend RangeQuery
  end

  class Set
    include Enumerable

    alias _each each
    alias _range range
    alias _starts_with starts_with
    alias _search_levenshtein search_levenshtein
    private :_each, :_range, :_starts_with, :_search_levenshtein

    prepend RangeQuery
  end
end
