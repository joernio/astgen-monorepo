# frozen_string_literal: true

require "ruby_ast_gen"

# `ParserProvider` loads prism lazily, and prism ships native extensions built for a single Java
# version, so it is genuinely unavailable on some JRuby/JDK combinations (the case PR #102 fixes).
# Load it up front so examples do not depend on each other for the constant, and let examples that
# need the real gem opt in with `requires_prism: true` so they can be skipped when it is missing.
PRISM_AVAILABLE =
  begin
    require "prism"
    require "prism/translation/parser"
    true
  rescue ScriptError, StandardError => e
    warn "[spec] prism unavailable (#{e.class}: #{e.message}); skipping prism-dependent examples"
    false
  end

RSpec.configure do |config|
  # Enable flags like --only-failures and --next-failure
  config.example_status_persistence_file_path = ".rspec_status"

  # Disable RSpec exposing methods globally on `Module` and `main`
  config.disable_monkey_patching!

  config.expect_with :rspec do |c|
    c.syntax = :expect
  end

  config.filter_run_excluding(requires_prism: true) unless PRISM_AVAILABLE
end
