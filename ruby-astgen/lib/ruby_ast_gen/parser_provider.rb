# frozen_string_literal: true

module RubyAstGen
  module ParserProvider

    def self.parse(buffer)
      begin
        require "prism"
        return Prism::Translation::Parser.new.parse(buffer)
      rescue LoadError, StandardError => e
        RubyAstGen::Logger::warn "Prism parser failed: #{e.class} - #{e.message}, trying whitequark parser gem"
      end

      begin
        require "parser/current"
        Parser::CurrentRuby.new.parse(buffer)
      rescue LoadError, StandardError => e
        RubyAstGen::Logger::error "Whitequark parser gem also failed: #{e.class} - #{e.message}"
        nil
      end
    end

  end
end
