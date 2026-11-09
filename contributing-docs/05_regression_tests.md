Regression tests
================

Unit tests tell you that the behaviour you wrote works. They do not tell you what else changed. The
[`astgen-regression`](../astgen-regression/README.md) framework answers that: it builds the astgen from your branch
and from the base branch, runs both on real-world open source projects (the "corpora"), diffs the outputs, and
reports the difference, the timing and any failures as a Markdown report.

Use it for any change that can alter the output of a parser: new fields, changed node shapes, a parser dependency
bump, performance work.

Where it is set up
------------------

| Tool | Config | CI workflow |
|------|--------|-------------|
| `javascript-astgen` | [regression.yaml](../javascript-astgen/regression.yaml) | `javascript-astgen-regression.yml` (Ubuntu) |
| `swift-astgen` | [regression.yaml](../swift-astgen/regression.yaml) | `swift-astgen-regression.yml` (macOS) |

Other tools do not have a regression setup yet, see [common tasks](08_common_tasks.md) to add one.

In CI the regression workflow runs on pull requests that touch the tool, it posts (and updates) a comment on the PR
with the report, and it **never fails the PR**. Regressions are for humans to judge: read the report, and explain
differences in the PR description.

Running it locally
------------------

```bash
cd astgen-regression
python -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt && pip install -e .

cd ../javascript-astgen          # or swift-astgen
astgen-regression local          # or: bun run regression
```

`local` needs git worktree support. It builds your working tree (so uncommitted changes are included), builds the base
branch (`main`) in a temporary git worktree, runs both on every corpus, cleans the worktree up, and prints the report.
The first run clones the corpora and takes a while.

Read the report as follows:

- **Artifacts changed** tells you which output files differ, with `N added, N removed, N changed`. Diffs are collapsed
  by default. `--output-diffs DIR` (on `compare`) writes full diffs to files.
- **Wall-clock time (median of N)** is measured with warmup runs and repeated runs, so noise is reduced but not gone.
  Prefer release builds for timing (Swift uses `swift build` in debug by default, which is slow).
- A very large single-file diff (over 20 000 normalized lines) is replaced with a placeholder, the summary is still
  accurate.

Configuration
-------------

`regression.yaml` in the tool directory has these sections: `project`, `build`, `execute`, `artifacts`, `corpora`,
`github`. The `execute.command` template uses `{dist_dir}`, `{input_dir}` and `{output_dir}`. The README of
`astgen-regression` documents every option.

When you add a corpus:

- Pin it to a **tag**, never a branch, so results are reproducible.
- Pick projects that exercise the language broadly and that are not huge. Every corpus runs `warmup + iterations`
  times for both base and PR (12 runs with the defaults).
- `execute.command` and the `build` commands run through a shell. Only put reviewed, trusted commands in the file.

Working on the framework itself
-------------------------------

`astgen-regression` is a Python 3.12 project with its own tests (`pytest tests/ -v`) and checks (`ruff`, `mypy`).
Read [astgen-regression/AGENTS.md](../astgen-regression/AGENTS.md) first. It documents the module split, the exit code
contract (`compare` exits 1 on regressions, and the generated workflow relies on that), the PR comment marker that
must stay in sync between `report.py` and `workflow.yml.j2`, and the error handling conventions.
