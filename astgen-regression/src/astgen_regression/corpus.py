"""Corpus cloning functionality."""

import subprocess
import sys
from pathlib import Path


def clone_corpus(corpus_config: dict, temp_dir: Path) -> Path | None:
    """Clone a git repository corpus.

    Args:
        corpus_config: Corpus configuration dict with keys:
            - name: Corpus identifier
            - clone_url: Git repository URL
            - tag: Git tag to checkout
            - input_subdir: Subdirectory to use as input (empty string = root)
        temp_dir: Temporary directory for cloning

    Returns:
        Path to input directory, or None on failure
    """
    name = corpus_config["name"]
    clone_url = corpus_config["clone_url"]
    tag = corpus_config["tag"]
    input_subdir = corpus_config.get("input_subdir", "")

    clone_dir = temp_dir / f"corpus-{name}"

    print(f"[corpus] Cloning {clone_url} @ {tag}...", file=sys.stderr)

    cmd = ["git", "clone", "--depth", "1", "--branch", tag, clone_url, str(clone_dir)]

    try:
        subprocess.run(
            cmd,
            capture_output=True,
            check=True,
            timeout=300,  # 5 minutes
        )
    except subprocess.CalledProcessError as e:
        stderr = e.stderr.decode(errors="replace")[:500]
        print(f"WARNING: Failed to clone {clone_url}: {stderr}", file=sys.stderr)
        return None
    except subprocess.TimeoutExpired:
        print(f"WARNING: Clone timeout for {clone_url} (5 minutes)", file=sys.stderr)
        return None
    except OSError as e:
        print(f"WARNING: Clone failed for {clone_url}: {e}", file=sys.stderr)
        return None

    # Determine input directory
    if input_subdir:
        input_path = clone_dir / input_subdir
    else:
        input_path = clone_dir

    return input_path
