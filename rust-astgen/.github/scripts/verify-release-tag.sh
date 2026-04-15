#!/usr/bin/env bash

# Verifies that the GitHub release tag (GITHUB_REF_NAME) matches the package version in Cargo.toml.

set -euo pipefail

crate_version="$(
  awk -F'"' '
    /^\[package\]/ { in_package = 1; next }
    /^\[/ { if (in_package) exit }
    in_package && /^version = / { print $2; exit }
  ' Cargo.toml
)"

if [[ -z "${crate_version}" ]]; then
  echo "::error title=Missing package version::Could not read the package version from Cargo.toml."
  exit 1
fi

if [[ -z "${GITHUB_REF_NAME:-}" ]]; then
  echo "::error title=Missing tag context::GITHUB_REF_NAME was not set."
  exit 1
fi

expected_tag="v${crate_version}"

if [[ "${GITHUB_REF_NAME}" != "${expected_tag}" ]]; then
  echo "::error title=Tag/version mismatch::Tag '${GITHUB_REF_NAME}' must match Cargo.toml version '${crate_version}' and use the format '${expected_tag}'."
  exit 1
fi
