Random tidbits
==============

### Build

- There is no root build. Every command needs the right working directory. If a command says it cannot find
  `package.json`, `Cargo.toml` or `go.mod`, you are in the wrong one.
- The first build of each tool is slow: dependencies are downloaded, and `rust-astgen` compiles the `rust-analyzer`
  crates. Later builds are incremental.
- `bun run test` and `bun check` are different things: tests versus type check. Run both. `bun check` needs
  `bun install` first.
- `bun run binary` cross-compiles six binaries and takes a while. For development run the sources:
  `bun src/astgen.ts`.
- Build output is git-ignored: `.build/`, `target/`, `node_modules/`, `build/`, `release/`, `vendor/`, and the
  renamed binaries (`astgen-linux`, `SwiftAstGen-mac`, ...). Still, do not `git add -A` blindly, check `git status`.

### Tests

- sbt 2 caches test results. Use `testFull`, not `test`, for the Scala round-trip suites: sbt cannot see that the
  generated Scala file or the astgen binary changed.
- The Scala suites of `javascript-astgen` and `swift-astgen` fail with a confusing error when the generated
  `*NodeSyntax.scala` or the astgen binary is missing. Check the prerequisites in [tests](04_tests.md).
- A test that fails only on one OS: check path separators, `\r\n` line endings, and file name case sensitivity.
  Go, Ruby, Rust and Swift run on all three platforms in CI.
- Rust tests that load the sysroot are slow. Use `--no-sysroot` unless the test is about `std` resolution.
- `ruby-astgen` tests run on MRI and JRuby. A gem that works on MRI may not on JRuby.

### Output

- Output must be deterministic. If the regression report shows noise (reordered keys or entries, different
  absolute paths, timestamps) the cause is nondeterminism in the tool, not in the report.
- The generated Scala file of Swift has a timestamp in its header, the rest is deterministic. The Rust file has no
  date when generated with `--include-date=false`, which is what CI compares.
- Absolute paths and the project root appear in some outputs (for example the Swift root node). Do not make a test
  depend on a machine-specific path.

### CI

- Docs-only PRs (`**/*.md`) skip all workflows, so a broken link or command in a README is never caught by CI.
  Run the commands you document.
- `detect-changes.js` diffs against `origin/{base branch}` and needs full git history (`fetch-depth: 0`). If you add a
  tool, its workflow also needs the paths in `monorepo-ci-config.yml`, otherwise it silently never runs.
- Release workflows only run for tags with the right prefix. Pushing `v1.2.3` without the `{tool}-astgen/` prefix
  does nothing.
- All releases are created as drafts and need to be published by hand.

### Misc

- Shared guidance for coding agents belongs in [AGENTS.md](../AGENTS.md), and `CLAUDE.md` is a symlink to it.
  `astgen-regression/` has its own `AGENTS.md` with `CLAUDE.md` as a symlink. Personal notes and per-directory
  hints can go into `CLAUDE.local.md` files and `.claude/settings.local.json`, which are git-ignored. Do not put
  anything in there that other contributors need.
- Windows checkouts need symlink support (`git config core.symlinks true`, and Developer Mode or admin rights) to see
  `CLAUDE.md` as a link. Without it, git checks it out as a text file containing `AGENTS.md`. Read `AGENTS.md`.
- The default branch is `main`.
