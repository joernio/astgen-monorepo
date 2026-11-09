Contributor's guide
===================

Contributions are welcome and are greatly appreciated! There are lots of ways to contribute to the Joern AST
generators, depending on your interests and skill level.

- Report bugs with a minimal reproducer, ideally a small code snippet in the affected language.
- Fix a parser bug: missing nodes, wrong ranges, crashes on valid input, wrong handling of some language construct.
- Add information that Joern frontends need (types, full names, resolved symbols) to the output.
- Improve the generated Scala accessors.
- Improve the regression testing framework or its corpora.
- Improve the documentation.
- Review pull requests.

Contents
--------

1. [Architecture](01_architecture.md): the tools, how they relate to Joern, repository layout, CI routing.
2. [Development environment](02_development_environment.md): toolchains and versions for each tool, IDE hints.
3. [Coding style](03_coding_style.md): formatters, linters and conventions per language.
4. [Tests](04_tests.md): unit tests, fixtures, and the Scala round-trip tests.
5. [Regression tests](05_regression_tests.md): comparing the output of a PR with `main` on real-world code.
6. [Releases](06_releases.md): versioning, tags and the release workflows.
7. [Pull requests](07_pull_requests.md): what a good PR looks like, and what CI checks.
8. [Common tasks](08_common_tasks.md): add a node or field, update a parser dependency, add a new tool.
99. [Hints](99_hints.md): pitfalls and tidbits.

Other documents
---------------

- [README](../README.md): overview of the tools and the release process.
- Each tool has its own README with usage and build instructions, for example
  [javascript-astgen/README.md](../javascript-astgen/README.md).
- [astgen-regression/README.md](../astgen-regression/README.md): the regression testing framework.

Reporting issues
----------------

Please open an [issue](https://github.com/joernio/astgen-monorepo/issues) and keep the conversation in English.
Share your input code or a minified example, the tool and its version (`--version`), and the output you expected.
That makes it much easier for us to help.

For AI coding agents
--------------------

The entry point for agents is [AGENTS.md](../AGENTS.md) in the repository root. `CLAUDE.md` is a symlink to it.
Keep `AGENTS.md` short, and put details in this directory.
[astgen-regression/AGENTS.md](../astgen-regression/AGENTS.md) holds additional guidance for that subproject.
