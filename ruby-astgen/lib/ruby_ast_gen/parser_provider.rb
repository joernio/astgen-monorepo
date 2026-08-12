# frozen_string_literal: true

module RubyAstGen
  module ParserProvider

    def self.parse(buffer)
      begin
        require "prism"
        return Prism::Translation::Parser.new.parse(buffer)
      rescue LoadError => e
        RubyAstGen::Logger::warn "Prism gem unavailable: #{e.class} - #{e.message}, trying whitequark parser gem"
      rescue StandardError => e
        RubyAstGen::Logger::warn "Prism parser failed: #{e.class} - #{e.message}, trying whitequark parser gem"
      end

      parse_with_whitequark(buffer)
    end

    def self.parse_with_whitequark(buffer)
      require "parser/current"
      Parser::CurrentRuby.new.parse(buffer)
    rescue LoadError, StandardError => e
      RubyAstGen::Logger::error "Whitequark parser gem also failed: #{e.class} - #{e.message}"
      nil
    end
    private_class_method :parse_with_whitequark

  end
end
