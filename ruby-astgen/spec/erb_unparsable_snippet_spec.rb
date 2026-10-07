# frozen_string_literal: true

require 'stringio'

RSpec.describe ErbToRubyTransformer do
  def capture_stdout
    original = $stdout
    $stdout = StringIO.new
    yield
    $stdout.string
  ensure
    $stdout = original
  end

  context "when a code snippet cannot be parsed" do
    before do
      allow(RubyAstGen::ParserProvider).to receive(:parse).and_return(nil)
    end

    it "emits `<%= x if y %>` unmodified as a template call and warns" do
      result = nil
      log = capture_stdout { result = described_class.new.transform("<p>a</p><%= foo if bar %>") }
      expect(result).to include("self.joernBufferAppend(self.joernBuffer, joernTemplateOutEscape( foo if bar ))")
      expect(log).to match(/\[WARN\] Unable to parse ERB code snippet.*foo if bar/)
    end

    it "uses the raw template call for `<%== x unless y %>`" do
      result = nil
      capture_stdout { result = described_class.new.transform("<%== foo unless bar %>") }
      expect(result).to include("joernBufferAppend(self.joernBuffer, joernTemplateOutRaw( foo unless bar ))")
    end

    it "emits an unparsable do-block snippet unmodified instead of dropping it" do
      result = nil
      capture_stdout { result = described_class.new.transform("<%= foo do |x| bar(x) end %>") }
      expect(result).to include("foo do |x| bar(x) end")
    end

    it "still lowers the surrounding content" do
      result = nil
      capture_stdout { result = described_class.new.transform("<h1>Hi</h1><%= foo if bar %><p>Bye</p>") }
      expect(result).to include("<h1>Hi</h1>")
      expect(result).to include("<p>Bye</p>")
    end
  end

  context "when snippets can be parsed" do
    it "does not warn" do
      log = capture_stdout { described_class.new.transform("<%= foo if bar %>") }
      expect(log).not_to include("Unable to parse ERB code snippet")
    end
  end
end
