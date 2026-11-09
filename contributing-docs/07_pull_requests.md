Pull requests
=============

Thank you for taking the time to contribute! This page describes what a pull request (PR) should look like, and what
CI checks, so that your PR can be merged quickly.

Before you start
----------------

- Check the [issue tracker](https://github.com/joernio/astgen-monorepo/issues) for an existing issue, or open one.
  For a larger change, or a change to the JSON output, discuss it in an issue first. Output changes affect the Joern
  frontends that consume it.
- For a bug fix, **reproduce the problem first**, ideally as a failing unit test.
- Read the [architecture](01_architecture.md) to find the tool that owns the problem.
- Keep a PR to **one tool**, unless the change is about CI, docs or something that really spans tools. Tools are
  built, tested and released separately, and CI runs per tool.

Checklist
---------

- [ ] The PR title starts with the tool directory in square brackets: `[javascript-astgen] Fix TSTypeCastExpression`.
      Use `[ci]` for workflow and CI configuration changes, `[docs]` for documentation, and
      `[astgen-regression]` for the regression framework. Repeat the prefix for changes to two tools:
      `[abap-astgen][javascript-astgen] Bump Bun to 1.4.3`.
- [ ] The description says what changed and why, and links the related issue. Do not leave it empty.
- [ ] If the JSON output changes, the description says how (new, renamed or removed keys) and whether Joern needs a
      matching change.
- [ ] A test verifies the change. For a bug fix it fails without the fix. See [tests](04_tests.md).
- [ ] The tool builds, its tests pass, and its formatter and type checker are clean. See
      [coding style](03_coding_style.md#run-before-you-submit).
- [ ] If you changed generated output (Scala accessors, version files), you regenerated it. For `rust-astgen` the
      committed `RustNodeSyntax.scala` must be up to date, CI checks it.
- [ ] If you changed a dependency, the lock file is updated (`bun.lock`, `Cargo.lock`, `Package.resolved`, `go.sum`).
- [ ] For parser changes that can alter output at scale: the [regression report](05_regression_tests.md) was reviewed.

Keep a PR focused. A PR that does one thing is reviewed faster than a large one. Do not include unrelated formatting
changes or refactorings.

Commit messages
---------------

The commit message (and the squashed PR title) has the same shape as the PR title and ends with the PR number that
GitHub adds:

```
[javascript-astgen] Generate Scala accessor (#177)
[ci] Run Swift/JS regression via monorepo-ci so docs-only PRs skip them (#183)
```

- Start with the tool directory in square brackets, for example `[rust-astgen]`.
- Use the imperative, and describe what changes, not how.

What CI checks
--------------

[monorepo-ci.yml](../.github/workflows/monorepo-ci.yml) runs on every PR. It calls only the workflows for the
directories you touched (see [architecture](01_architecture.md#ci-routing)), so a PR that touches
`rust-astgen/` does not run the Swift workflow.

| Workflow | What it does | Reproduce locally |
|----------|--------------|-------------------|
| `abap-astgen-ci.yml` | type check, tests, builds binaries | `bun install && bun check && bun run test` |
| `javascript-astgen-ci.yml` | type check, tests, builds binaries, Scala tests | see [tests](04_tests.md) |
| `javascript-astgen-regression.yml` | regression run on real-world code, comments on the PR | `bun run regression` |
| `dotnet-astgen-ci.yml` | build, tests, `dotnet format` check | `dotnet test && dotnet format --verify-no-changes` |
| `go-astgen-ci.yml` | build and tests on three OSes | `./build.sh` |
| `ruby-astgen-ci.yml` | specs on MRI and JRuby on three OSes | `bundle exec rake` |
| `rust-astgen-ci.yml` | tests on three OSes, bindings check, Scala tests, `cargo fmt --check` | see [tests](04_tests.md) |
| `swift-astgen-ci.yml` | build, tests, `swift-format lint`, Scala tests on three OSes | `swift test` |
| `swift-astgen-regression.yml` | regression run on real-world code (macOS) | `astgen-regression local` |
| `astgen-regression-ci.yml` | `ruff`, `mypy`, `pytest` | see [coding style](03_coding_style.md) |

Special cases:

- A PR that only touches Markdown files (`**/*.md`), `docs/`, `LICENSE` or `.gitignore` runs **no** workflows.
- A PR that touches `monorepo-ci.yml` or `monorepo-ci-config.yml` runs **all** workflows.
- When a job fails on a platform you do not have, read the log first. Typical causes are path separators, line
  endings, file name case, an unordered collection, or a tool version that differs from the runner.

Dependabot opens weekly PRs that bump the GitHub Actions. Actions are pinned by commit SHA with the version in a
trailing comment. Keep that format.

Review
------

- If your PR goes stale, follow up in a comment and ask about the priority.
- Respond to review comments with a new commit, and mention what you changed. The PR is squashed on merge.

AI-assisted changes
-------------------

The same rules apply to changes written with the help of an AI tool. You are responsible for what you submit:
read and understand every line, run the checks above yourself, and do not submit code you cannot explain.
Do not reproduce copyrighted code.
