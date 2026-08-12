#!/usr/bin/env bash
set -euo pipefail

run_jruby() {
  if [[ -n "${JRUBY_JAR:-}" && -f "${JRUBY_JAR}" ]]; then
    java -jar "${JRUBY_JAR}" "$@"
  else
    jruby "$@"
  fi
}

RUBY_ABI="$(run_jruby -e 'print Gem.ruby_api_version')"
GEM_HOME="${GEM_HOME:-${PWD}/vendor/bundle/jruby/${RUBY_ABI}}"
EXTENSIONS_DIR="${GEM_HOME}/extensions"
ROOT="${PWD}"

if [[ ! -d "${EXTENSIONS_DIR}" ]]; then
  echo "No prism extensions directory at ${EXTENSIONS_DIR}; run bundle install first" >&2
  exit 1
fi

run_fallback_scenario() {
  local scenario_name="$1"
  local setup_fn="$2"

  local backup_dir
  backup_dir="$(mktemp -d)"

  find "${EXTENSIONS_DIR}" -mindepth 1 -maxdepth 1 -exec mv {} "${backup_dir}/" \;
  "${setup_fn}" "${backup_dir}"

  local output
  output="$(
    RUBY_ASTGEN_ROOT="${ROOT}" \
    GEM_HOME="${GEM_HOME}" \
    GEM_PATH="${GEM_HOME}" \
    run_jruby <<'RUBY'
require "fileutils"
$LOAD_PATH.unshift(File.join(ENV.fetch("RUBY_ASTGEN_ROOT"), "lib"))
require "ruby_ast_gen"

buffer = Parser::Source::Buffer.new("test.rb")
buffer.source = "class Foo; end"
ast = RubyAstGen::ParserProvider.parse(buffer)
abort("expected fallback parse to succeed") unless ast&.type == :class
RUBY
  )"

  find "${EXTENSIONS_DIR}" -mindepth 1 -maxdepth 1 -exec rm -rf {} + 2>/dev/null || true
  find "${backup_dir}" -mindepth 1 -maxdepth 1 -exec mv {} "${EXTENSIONS_DIR}/" \;
  rmdir "${backup_dir}"

  echo "${output}"

  if ! grep -q "\[WARN\] Prism gem unavailable: LoadError" <<<"${output}"; then
    echo "Expected prism LoadError warning was not logged for ${scenario_name}" >&2
    exit 1
  fi
}

remove_all_extensions() {
  :
}

install_mismatched_platform_extensions() {
  local backup_dir="$1"
  local current_platform
  current_platform="$(run_jruby -e 'print Gem.platforms.grep(Gem::Platform).find { |p| p.os == "java" }')"
  local mismatched_platform
  if [[ "${current_platform}" == *"-21" ]]; then
    mismatched_platform="universal-java-25"
  else
    mismatched_platform="universal-java-21"
  fi

  local source_platform
  source_platform="$(find "${backup_dir}" -mindepth 1 -maxdepth 1 -type d | head -n 1)"
  mkdir -p "${EXTENSIONS_DIR}/${mismatched_platform}"
  cp -R "${source_platform}/." "${EXTENSIONS_DIR}/${mismatched_platform}/"
}

run_fallback_scenario "missing prism extensions" remove_all_extensions
run_fallback_scenario "mismatched Java platform extensions" install_mismatched_platform_extensions

echo "Prism fallback integration tests passed"
