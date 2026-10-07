# frozen_string_literal: true

require 'tmpdir'
require 'fileutils'
require 'json'
require 'stringio'

RSpec.describe "ERB lowering under concurrency" do
  MAILER_ERB = <<~ERB
    <!DOCTYPE html>
    <html>
      <body>
        <%= yield %>
      </body>
    </html>
  ERB

  def run_astgen(dir, out)
    RubyAstGen::parse(input: dir, output: out, exclude: '^(tests?|vendor|spec)', debug: false)
  end

  it "lowers every ERB file identically when processed by many threads" do
    Dir.mktmpdir do |dir|
      Dir.mktmpdir do |out|
        40.times do |i|
          FileUtils.mkdir_p(File.join(dir, "views#{i}"))
          File.write(File.join(dir, "views#{i}", "mailer.html.erb"), MAILER_ERB)
        end

        run_astgen(dir, out)

        jsons = Dir.glob(File.join(out, "**", "mailer.html.erb.json"))
        expect(jsons.size).to eq(40)
        normalized = jsons.map do |f|
          JSON.parse(File.read(f)).tap { |j| j.delete("file_path"); j.delete("rel_file_path") }
        end
        expect(normalized.uniq.size).to eq(1)
        expect(File.read(jsons.first)).to include("joernBufferAppend")
        expect(File.read(jsons.first)).not_to include("HEREDOC")
      end
    end
  end

  it "reports a warning that names the file when lowering falls back to raw content" do
    allow_any_instance_of(ErbToRubyTransformer).to receive(:transform).and_raise(StandardError, "boom")
    output = StringIO.new
    original = $stdout
    $stdout = output
    begin
      RubyAstGen.get_erb_content(MAILER_ERB, "app/views/layouts/mailer.html.erb")
    ensure
      $stdout = original
    end
    expect(output.string).to match(/\[WARN\] Failed to lower ERB .* - app\/views\/layouts\/mailer\.html\.erb/)
  end
end
