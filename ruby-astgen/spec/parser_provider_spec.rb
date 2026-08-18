# frozen_string_literal: true

RSpec.describe RubyAstGen::ParserProvider do
  # `prism_available?` memoises its result across calls, so reset it between examples.
  before { described_class.instance_variable_set(:@prism_available, nil) }

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
      expect(described_class.parse(buffer_for(""))).to be_nil
    end
  end

  describe "when the prism gem cannot be loaded" do
    before do
      allow(described_class).to receive(:require_prism)
        .and_raise(LoadError, "cannot load such file -- prism")
    end

    it "lets the LoadError propagate so build problems surface loudly" do
      expect { described_class.parse(buffer_for("class Foo; end")) }
        .to raise_error(LoadError, /prism/)
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

    it "logs a warning and falls back when prism crashes on invalid syntax" do
      expect(RubyAstGen::Logger).to receive(:warn).with(/\APrism parser failed: /)

      # The parser gem also fails on this input, but silently (it returns nil).
      expect(described_class.parse(buffer_for("def class end end }{]["))).to be_nil
    end
  end

  describe "when both parsers fail" do
    it "logs an error and returns nil when prism and whitequark both raise" do
      allow(Prism::Translation::Parser).to receive(:new).and_raise(StandardError, "prism error")
      allow(Parser::CurrentRuby).to receive(:new).and_raise(StandardError, "parser error")

      expect(RubyAstGen::Logger).to receive(:warn)
        .with("Prism parser failed: StandardError - prism error, trying whitequark parser gem")
      expect(RubyAstGen::Logger).to receive(:error)
        .with("Whitequark parser gem also failed: StandardError - parser error")

      expect(described_class.parse(buffer_for("class Foo; end"))).to be_nil
    end
  end
end
