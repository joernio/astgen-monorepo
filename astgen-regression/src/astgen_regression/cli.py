"""CLI argument parsing and subcommand dispatch."""

import argparse
from pathlib import Path
import sys

from astgen_regression.commands.init import LANGUAGE_DEFAULTS


def create_parser() -> argparse.ArgumentParser:
    """Create argument parser with subcommands.

    Returns:
        Configured ArgumentParser
    """
    parser = argparse.ArgumentParser(
        prog="astgen-regression",
        description="Config-driven regression testing for astgen variants"
    )

    subparsers = parser.add_subparsers(dest="command", required=True)

    # init subcommand
    init_parser = subparsers.add_parser(
        "init",
        help="Initialize regression testing (generate config and workflow)"
    )
    init_parser.add_argument(
        "--language",
        choices=list(LANGUAGE_DEFAULTS.keys()),
        help="Pre-fill config for language"
    )
    init_parser.add_argument(
        "--config",
        type=Path,
        default=Path("regression.yaml"),
        help="Config file path (default: regression.yaml)"
    )
    init_parser.add_argument(
        "--force",
        action="store_true",
        help="Overwrite existing files"
    )

    # compare subcommand
    compare_parser = subparsers.add_parser(
        "compare",
        help="Compare two pre-built distributions"
    )
    compare_parser.add_argument(
        "--base-dist",
        type=Path,
        required=True,
        help="Path to base distribution directory"
    )
    compare_parser.add_argument(
        "--pr-dist",
        type=Path,
        required=True,
        help="Path to PR distribution directory"
    )
    compare_parser.add_argument(
        "--config",
        type=Path,
        default=Path("regression.yaml"),
        help="Config file path (default: regression.yaml)"
    )
    compare_parser.add_argument(
        "--pr-number",
        type=int,
        help="PR number for report metadata"
    )
    compare_parser.add_argument(
        "--base-ref",
        help="Human-readable base ref (e.g. 'main @ abc1234')"
    )
    compare_parser.add_argument(
        "--pr-ref",
        help="Human-readable PR ref (e.g. 'feature @ def5678')"
    )
    compare_parser.add_argument(
        "--output-diffs",
        type=Path,
        help="Directory to write full diff files"
    )

    # local subcommand
    local_parser = subparsers.add_parser(
        "local",
        help="Run regression locally using git worktrees"
    )
    local_parser.add_argument(
        "--base-branch",
        help="Base branch to compare against (default: from config or 'main')"
    )
    local_parser.add_argument(
        "--config",
        type=Path,
        default=Path("regression.yaml"),
        help="Config file path (default: regression.yaml)"
    )

    return parser


def main() -> None:
    """Main CLI entry point."""
    parser = create_parser()
    args = parser.parse_args()

    if args.command == "init":
        from astgen_regression.commands.init import cmd_init
        cmd_init(args)
    elif args.command == "compare":
        from astgen_regression.commands.compare import cmd_compare
        cmd_compare(args)
    elif args.command == "local":
        from astgen_regression.commands.local import cmd_local
        cmd_local(args)
    else:
        parser.print_help()
        sys.exit(1)
