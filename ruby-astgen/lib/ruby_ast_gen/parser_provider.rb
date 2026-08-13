# frozen_string_literal: true

module RubyAstGen
  module ParserProvider

    def self.parse(buffer)
      if prism_available?
        begin
          return Prism::Translation::Parser.new.parse(buffer)
        rescue StandardError => e
          RubyAstGen::Logger::warn "Prism parser failed: #{e.class} - #{e.message}, trying whitequark parser gem"
        end
      end

      parse_with_whitequark(buffer)
    end

    # Memoised because a failed `require` is not recorded in `$LOADED_FEATURES`: without this,
    # every parsed file would re-scan the load path and re-log the same warning.
    def self.prism_available?
      return @prism_available unless @prism_available.nil?

      require_prism
      @prism_available = true
    rescue ScriptError, StandardError => e
      RubyAstGen::Logger::warn "Prism gem unavailable: #{e.class} - #{e.message}, using whitequark parser gem"
      @prism_available = false
    end

    # Extracted so specs can simulate the gem being unavailable, e.g. when the shipped native
    # extensions were built for a different Java version than the one JRuby is running on.
    def self.require_prism
      require "prism"
    end

    def self.parse_with_whitequark(buffer)
      require "parser/current"
      Parser::CurrentRuby.new.parse(buffer)
    rescue ScriptError, StandardError => e
      RubyAstGen::Logger::error "Whitequark parser gem also failed: #{e.class} - #{e.message}"
      nil
    end

    private_class_method :prism_available?, :require_prism, :parse_with_whitequark

  end
end
