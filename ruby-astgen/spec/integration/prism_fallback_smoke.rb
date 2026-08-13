# frozen_string_literal: true

# Smoke test for the parser fallback chain against a real JRuby/JDK combination.
#
# prism ships native extensions built for a single Java version, so running a release under a
# different JDK makes `require "prism"` raise LoadError. When that happens we must fall back to the
# whitequark parser gem instead of letting the error escape to the caller.
#
# Run it *without* bundler, the way a released gem is loaded: when prism's extensions do not match
# the running platform, `Bundler.setup` refuses to start the process at all, so `bundle exec` cannot
# reach this code. RubyGems on its own just ignores the gem, which is the situation we handle.
#
#   GEM_HOME=<bundle path> GEM_PATH=<bundle path> jruby -Ilib spec/integration/prism_fallback_smoke.rb
#
# Set EXPECT_PRISM=true/false to assert which parser served the request, so a run cannot pass
# vacuously by silently testing the wrong scenario.
require "ruby_ast_gen"

buffer = Parser::Source::Buffer.new("smoke.rb")
buffer.source = <<~RUBY
  class Foo
    def bar(x)
      x + 1
    end
  end
RUBY

ast = RubyAstGen::ParserProvider.parse(buffer)
abort "expected a parsed AST of type :class, got #{ast.inspect}" unless ast&.type == :class

# Reported so the CI log shows which parser actually served the request, rather than leaving it
# ambiguous whether the fallback path was exercised at all.
prism_available = RubyAstGen::ParserProvider.send(:prism_available?)
puts "parser fallback smoke test passed on #{RUBY_DESCRIPTION}"
puts "prism available: #{prism_available}"

expected = ENV["EXPECT_PRISM"]
if expected && expected != prism_available.to_s
  abort "expected prism availability to be #{expected}, got #{prism_available}"
end
