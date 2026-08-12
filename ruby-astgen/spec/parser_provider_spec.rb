# frozen_string_literal: true

RSpec.describe RubyAstGen::ParserProvider do
  def buffer_for(source, name = "test.rb")
    buffer = Parser::Source::Buffer.new(name)
    buffer.source = source
    buffer
  end

  describe "when prism is available" do
    it "parses valid Ruby with prism" do
      ast = described_class.parse(buffer_for("class Foo; end"))
      expect(ast).not_to be_nil
      expect(ast.type).to eq(:class)
    end

    it "returns nil for empty source without falling back to the parser gem" do
      expect(RubyAstGen::Logger).not_to receive(:warn)
      ast = described_class.parse(buffer_for(""))
      expect(ast).to be_nil
    end
  end

  describe "when prism is loaded but fails while parsing" do
    it "logs a warning and falls back to the parser gem on StandardError" do
      expect(RubyAstGen::Logger).to receive(:warn)
        .with("Prism parser failed: StandardError - prism error, trying whitequark parser gem")
      allow(Prism::Translation::Parser).to receive(:new).and_raise(StandardError, "prism error")

      ast = described_class.parse(buffer_for("class Foo; end"))
      expect(ast).not_to be_nil
      expect(ast.type).to eq(:class)
    end

    it "logs a warning and falls back when prism crashes with NoMethodError on invalid syntax" do
      expect(RubyAstGen::Logger).to receive(:warn).with(/Prism parser failed: NoMethodError/)

      ast = described_class.parse(buffer_for("def class end end }{]["))
      expect(ast).to be_nil
    end
  end

  describe "when both parsers fail" do
    it "logs an error and returns nil when the parser gem raises StandardError" do
      expect(RubyAstGen::Logger).to receive(:warn)
        .with("Prism parser failed: StandardError - prism error, trying whitequark parser gem")
      expect(RubyAstGen::Logger).to receive(:error)
        .with("Whitequark parser gem also failed: StandardError - parser error")

      allow(Prism::Translation::Parser).to receive(:new).and_raise(StandardError, "prism error")
      allow(Parser::CurrentRuby).to receive(:new).and_raise(StandardError, "parser error")

      ast = described_class.parse(buffer_for("class Foo; end"))
      expect(ast).to be_nil
    end

    it "logs an error and returns nil when the parser gem raises LoadError" do
      expect(RubyAstGen::Logger).to receive(:warn)
        .with("Prism parser failed: StandardError - prism error, trying whitequark parser gem")
      expect(RubyAstGen::Logger).to receive(:error)
        .with("Whitequark parser gem also failed: LoadError - parser gem unavailable")

      allow(Prism::Translation::Parser).to receive(:new).and_raise(StandardError, "prism error")
      allow(Parser::CurrentRuby).to receive(:new).and_raise(LoadError, "parser gem unavailable")

      ast = described_class.parse(buffer_for("class Foo; end"))
      expect(ast).to be_nil
    end
  end

  describe "LoadError handling contract" do
    it "uses a distinct warning when the prism gem cannot be required" do
      source = File.read(File.expand_path("../lib/ruby_ast_gen/parser_provider.rb", __dir__))
      expect(source).to include("rescue LoadError => e")
      expect(source).to include("Prism gem unavailable")
    end
  end
end
