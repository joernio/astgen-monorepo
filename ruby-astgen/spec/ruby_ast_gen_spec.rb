# frozen_string_literal: true

require 'tempfile'

RSpec.describe RubyAstGen do
  temp_name = ""
  let(:temp_file) {
    file = Tempfile.new('test_ruby_code')
    temp_name = File.basename(file.path)
    file
  }

  after(:each) do
    temp_file.close
    temp_file.unlink
  end

  def code(s)
    temp_file.write(s)
    temp_file.rewind
  end

  it "should parse a class successfully" do
    code(<<-CODE)
class Foo
  CONST = 1
end
    CODE
    ast = RubyAstGen::parse_file(temp_file.path, temp_name)
    expect(ast).not_to be_nil
  end

  it "should parse assignment to HEREDOCs successfully" do
    code(<<-CODE)
multi_line_string = <<-TEXT
This is a multi-line string.
You can freely write across
multiple lines using heredoc.
TEXT
    CODE
    ast = RubyAstGen::parse_file(temp_file.path, temp_name)
    expect(ast).not_to be_nil
  end


  it "should parse call with HEREDOC args successfully" do
    code(<<-CODE)
puts(<<-ARG1, <<-ARG2)
This is the first HEREDOC.
It spans multiple lines.
ARG1
This is the second HEREDOC.
It also spans multiple lines.
ARG2
    CODE
    ast = RubyAstGen::parse_file(temp_file.path, temp_name)
    expect(ast).not_to be_nil
  end

  it "should create a singleton object body successfully" do
    code(<<-CODE)
class C
 class << self
  def f(x)
   x + 1
  end
 end
end
    CODE
    ast = RubyAstGen::parse_file(temp_file.path, temp_name)
    expect(ast).not_to be_nil
  end

  it "should create an operator assignment successfully" do
    code(<<-CODE)
def foo(x)
  x += 1
end
    CODE
    ast = RubyAstGen::parse_file(temp_file.path, temp_name)
    expect(ast).not_to be_nil
  end
end
