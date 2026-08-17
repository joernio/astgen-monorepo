# frozen_string_literal: true

# Smoke test for prism loading against a real JRuby/JDK combination.
#
# Prism ships native extensions built for a single Java version, and RubyGems refuses to load the
# gem when the runtime JDK differs. require_prism works around this by adding prism's lib/ directory
# directly to $LOAD_PATH, so prism should load regardless of JDK version.
#
# Run it *without* bundler: when prism's extensions do not match the running platform,
# `Bundler.setup` aborts before any of our code runs, so `bundle exec` cannot reach this code.
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
