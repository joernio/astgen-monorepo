# frozen_string_literal: true

require "open3"
require "rbconfig"
require "tmpdir"

RSpec.describe "exe/ruby_ast_gen" do
  let(:exe) { File.expand_path("../exe/ruby_ast_gen", __dir__) }

  it "does not consume the argument after -d" do
    Dir.mktmpdir do |dir|
      input = File.join(dir, "src")
      output = File.join(dir, "out")
      Dir.mkdir(input)
      File.write(File.join(input, "foo.rb"), "class Foo; end\n")

      out, status = Open3.capture2e(RbConfig.ruby, exe, "-d", "-i", input, "-o", output)

      expect(status.success?).to be(true), out
      expect(Dir.glob(File.join(output, "**", "*.json"))).not_to be_empty
    end
  end
end
