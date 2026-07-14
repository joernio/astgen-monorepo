# frozen_string_literal: true

module RubyAstGen
  module ParserProvider

    def self.parse(buffer)
      begin
        require "prism"
        return Prism::Translation::Parser.new.parse(buffer)
      rescue StandardError => e
        RubyAstGen::Logger::warn "Prism parser failed: #{e.class} - #{e.message}, trying parser gem"
      end

      begin
        require "parser/current"
        Parser::CurrentRuby.new.parse(buffer)
      rescue StandardError => e
        RubyAstGen::Logger::error "Parser gem also failed: #{e.class} - #{e.message}"
        nil
      end
    end

    def self.new_parser
      require "prism"
      Prism::Translation::Parser.new
    rescue LoadError
      require "parser/current"
      RubyAstGen::Logger::warn "Prism gem not available, using parser gem"
      Parser::CurrentRuby.new
    end

  end
end
