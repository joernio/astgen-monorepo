# frozen_string_literal: true

RSpec.describe RubyAstGen::ParserProvider do
  # `prism_available?` memoises its result across calls, so reset it between examples.
  before { described_class.instance_variable_set(:@prism_available, nil) }

  def buffer_for(source, name = "test.rb")
    buffer = Parser::Source::Buffer.new(name)
    buffer.source = source
    buffer
  end

  describe "when prism is available", requires_prism: true do
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

    it "logs a distinct warning and falls back to the parser gem" do
      expect(RubyAstGen::Logger).to receive(:warn)
        .with("Prism gem unavailable: LoadError - cannot load such file -- prism, using whitequark parser gem")

      ast = described_class.parse(buffer_for("class Foo; end"))
      expect(ast).not_to be_nil
      expect(ast.type).to eq(:class)
    end

    it "warns and retries the require only once, however many files are parsed" do
      expect(RubyAstGen::Logger).to receive(:warn).once

      3.times do
        expect(described_class.parse(buffer_for("class Foo; end")).type).to eq(:class)
      end

      expect(described_class).to have_received(:require_prism).once
    end
  end

  describe "when prism is loaded but fails while parsing", requires_prism: true do
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
    it "logs an error and returns nil when prism is missing and the parser gem raises" do
      allow(described_class).to receive(:require_prism)
        .and_raise(LoadError, "cannot load such file -- prism")
      allow(Parser::CurrentRuby).to receive(:new).and_raise(StandardError, "parser error")

      expect(RubyAstGen::Logger).to receive(:warn)
        .with("Prism gem unavailable: LoadError - cannot load such file -- prism, using whitequark parser gem")
      expect(RubyAstGen::Logger).to receive(:error)
        .with("Whitequark parser gem also failed: StandardError - parser error")

      expect(described_class.parse(buffer_for("class Foo; end"))).to be_nil
    end

    it "logs an error and returns nil when the parser gem raises LoadError", requires_prism: true do
      allow(Prism::Translation::Parser).to receive(:new).and_raise(StandardError, "prism error")
      allow(Parser::CurrentRuby).to receive(:new).and_raise(LoadError, "parser gem unavailable")

      expect(RubyAstGen::Logger).to receive(:warn)
        .with("Prism parser failed: StandardError - prism error, trying whitequark parser gem")
      expect(RubyAstGen::Logger).to receive(:error)
        .with("Whitequark parser gem also failed: LoadError - parser gem unavailable")

      expect(described_class.parse(buffer_for("class Foo; end"))).to be_nil
    end

    # `LoadError` and `NotImplementedError` are both `ScriptError`, not `StandardError`: the bug this
    # guards against is a bare `rescue StandardError` letting them escape to the caller.
    it "logs an error and returns nil when the parser gem raises a non-StandardError ScriptError" do
      allow(described_class).to receive(:require_prism)
        .and_raise(LoadError, "cannot load such file -- prism")
      allow(Parser::CurrentRuby).to receive(:new).and_raise(NotImplementedError, "no parser here")

      expect(RubyAstGen::Logger).to receive(:warn)
        .with("Prism gem unavailable: LoadError - cannot load such file -- prism, using whitequark parser gem")
      expect(RubyAstGen::Logger).to receive(:error)
        .with("Whitequark parser gem also failed: NotImplementedError - no parser here")

      expect { described_class.parse(buffer_for("class Foo; end")) }.not_to raise_error
    end
  end
end
