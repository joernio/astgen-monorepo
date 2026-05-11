"""Compare command implementation."""

from pathlib import Path
import tempfile
import shutil
import sys

from astgen_regression.config import load_config
from astgen_regression.corpus import clone_corpus
from astgen_regression.executor import execute_astgen_repeated
from astgen_regression.metrics import collect_metrics
from astgen_regression.compare import compare_outputs
from astgen_regression.report import render_report, write_diff_files


def cmd_compare(args) -> None:
    """Run compare command.

    Args:
        args: Parsed command-line arguments
    """
    # Load config
    try:
        config = load_config(args.config)
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)

    base_dist = args.base_dist
    pr_dist = args.pr_dist

    if not base_dist.exists():
        print(f"ERROR: Base dist not found: {base_dist}", file=sys.stderr)
        sys.exit(1)

    if not pr_dist.exists():
        print(f"ERROR: PR dist not found: {pr_dist}", file=sys.stderr)
        sys.exit(1)

    # Create temp directory for corpora
    temp_dir = tempfile.mkdtemp(prefix="astgen-regression-")
    corpus_results = []

    try:
        artifacts_config = config.get(
            "artifacts", [{"name": "ast", "pattern": "*.json"}]
        )

        execute_config = config["execute"]
        iterations = int(execute_config.get("iterations", 5))
        warmup = int(execute_config.get("warmup", 1))

        for corpus_config in config["corpora"]:
            name = corpus_config["name"]
            label = corpus_config["label"]

            print(f"[regression] Processing corpus: {name}", file=sys.stderr)

            # Clone corpus
            input_dir = clone_corpus(corpus_config, Path(temp_dir))

            if input_dir is None:
                # Clone failed
                corpus_results.append(
                    {
                        "name": name,
                        "label": f"{label} [CLONE FAILED]",
                        "base_metrics": {},
                        "pr_metrics": {},
                        "base_time": 0.0,
                        "pr_time": 0.0,
                        "iterations": iterations,
                        "comparison": {
                            "only_in_base": [],
                            "only_in_pr": [],
                            "diffs": {},
                        },
                    }
                )
                continue

            base_output = Path(temp_dir) / f"out-base-{name}"
            print(f"[regression] Running base astgen on {name}...", file=sys.stderr)
            _, base_time, _ = execute_astgen_repeated(
                execute_config,
                base_dist,
                input_dir,
                base_output,
                iterations=iterations,
                warmup=warmup,
                label=f"{name}/base",
            )

            pr_output = Path(temp_dir) / f"out-pr-{name}"
            print(f"[regression] Running PR astgen on {name}...", file=sys.stderr)
            _, pr_time, _ = execute_astgen_repeated(
                execute_config,
                pr_dist,
                input_dir,
                pr_output,
                iterations=iterations,
                warmup=warmup,
                label=f"{name}/pr",
            )

            # Output of the last timed run is what we compare against.
            base_metrics = collect_metrics(base_output, artifacts_config)
            pr_metrics = collect_metrics(pr_output, artifacts_config)

            print(f"[regression] Comparing outputs for {name}...", file=sys.stderr)
            comparison = compare_outputs(base_output, pr_output, artifacts_config)

            corpus_results.append(
                {
                    "name": name,
                    "label": label,
                    "base_metrics": base_metrics,
                    "pr_metrics": pr_metrics,
                    "base_time": base_time,
                    "pr_time": pr_time,
                    "iterations": iterations,
                    "comparison": comparison,
                }
            )

    finally:
        # Clean up temp directory
        shutil.rmtree(temp_dir, ignore_errors=True)

    # Generate report
    metadata = {
        "project_name": config["project"]["name"],
        "base_ref": args.base_ref,
        "pr_ref": args.pr_ref,
        "pr_number": args.pr_number,
    }

    report = render_report(corpus_results, artifacts_config, metadata)

    # Write diff files if requested
    if args.output_diffs:
        write_diff_files(args.output_diffs, corpus_results, artifacts_config)

    # Print report to stdout
    print(report)

    # Exit with code 1 if regressions detected
    has_regressions = any(
        len(result["comparison"]["diffs"].get(art["name"], [])) > 0
        for result in corpus_results
        for art in artifacts_config
    )

    if has_regressions:
        sys.exit(1)
