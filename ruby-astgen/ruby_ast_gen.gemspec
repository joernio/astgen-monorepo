# frozen_string_literal: true

require_relative "lib/ruby_ast_gen/version"

Gem::Specification.new do |spec|
  spec.name = "ruby_ast_gen"
  spec.version = RubyAstGen::VERSION
  spec.authors = ["David Baker Effendi"]
  spec.email = ["dave@whirlylabs.com"]

  spec.summary = "A Ruby parser than dumps the AST as JSON output"
  spec.description = "A Ruby parser than dumps the AST as JSON output for Joern's `rubysrc2cpg` frontend"
  spec.homepage = "https://github.com/whirlylabs/ruby_ast_gen"
  spec.license = "MIT"
  spec.required_ruby_version = ">= 3.0.0"

  # spec.metadata["allowed_push_host"] = "TODO: Set to your gem server 'https://example.com'"

  spec.metadata["homepage_uri"] = spec.homepage
  # spec.metadata["source_code_uri"] = "TODO: Put your gem's public repo URL here."
  # spec.metadata["changelog_uri"] = "TODO: Put your gem's CHANGELOG.md URL here."

  # Specify which files should be added to the gem when it is released.
  # The `git ls-files -z` loads the files in the RubyGem that have been added into git.
  gemspec = File.basename(__FILE__)
  spec.files = IO.popen(%w[git ls-files -z], chdir: __dir__, err: IO::NULL) do |ls|
    ls.readlines("\x0", chomp: true).reject do |f|
      (f == gemspec) ||
        f.start_with?(*%w[bin/ test/ spec/ features/ .git .github appveyor Gemfile])
    end
  end
  spec.bindir = "exe"
  spec.executables = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]

  spec.add_dependency 'parser', '~> 3.3', '>= 3.3.5.0'
  spec.add_dependency 'slop', '~> 4.10', '>= 4.10.1'
  spec.add_dependency 'logger', '~> 1.6', '>= 1.6'
  spec.add_dependency 'ostruct', '~> 0.6.0'
end
