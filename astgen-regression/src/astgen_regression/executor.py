"""Astgen execution functionality."""

from pathlib import Path
import shutil
import statistics
import subprocess
import sys
import time


def render_command(
    exec_config: dict, dist_dir: Path, input_dir: Path, output_dir: Path
) -> str:
    """Render command template with placeholders replaced.

    Args:
        exec_config: Execute configuration with 'command' template
        dist_dir: Distribution directory path
        input_dir: Input directory path
        output_dir: Output directory path

    Returns:
        Rendered command string
    """
    cmd_template = exec_config["command"]
    return cmd_template.format(
        dist_dir=str(dist_dir), input_dir=str(input_dir), output_dir=str(output_dir)
    )


def execute_astgen(
    exec_config: dict, dist_dir: Path, input_dir: Path, output_dir: Path
) -> tuple[bool, float]:
    """Execute astgen binary once and measure elapsed time.

    The output directory is created if it does not exist; existing contents are
    left in place. Use ``execute_astgen_repeated`` if you need a clean run each
    time or want median-of-N timing.

    Args:
        exec_config: Execute configuration with 'command' and 'timeout'
        dist_dir: Distribution directory path
        input_dir: Input directory path
        output_dir: Output directory path

    Returns:
        Tuple of (success: bool, elapsed_seconds: float)
    """
    cmd_str = render_command(exec_config, dist_dir, input_dir, output_dir)
    timeout = exec_config.get("timeout", 600)

    output_dir.mkdir(parents=True, exist_ok=True)

    t0 = time.monotonic()

    try:
        result = subprocess.run(
            cmd_str,
            shell=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout,
        )
        elapsed = time.monotonic() - t0

        if result.returncode != 0:
            stderr = result.stderr.decode(errors="replace")[:500]
            print(
                f"WARNING: astgen exited {result.returncode}\n  stderr: {stderr}",
                file=sys.stderr,
            )
            return False, elapsed

        return True, elapsed

    except subprocess.TimeoutExpired:
        elapsed = time.monotonic() - t0
        print(f"WARNING: astgen timed out after {elapsed:.1f}s", file=sys.stderr)
        return False, elapsed

    except Exception as e:
        elapsed = time.monotonic() - t0
        print(f"WARNING: astgen execution failed: {e}", file=sys.stderr)
        return False, elapsed


def execute_astgen_repeated(
    exec_config: dict,
    dist_dir: Path,
    input_dir: Path,
    output_dir: Path,
    iterations: int = 5,
    warmup: int = 1,
    label: str | None = None,
) -> tuple[bool, float, list[float]]:
    """Execute astgen ``warmup + iterations`` times and return the median time.

    The output directory is wiped before every run (warmup runs included) so each
    iteration measures the same end-to-end work. Warmup runs are not included in
    the returned timings; they exist to prime CPU/file-system caches.

    On the first failed run the function stops immediately and returns
    ``(False, ...)``. Per-iteration timings are also written to stderr to make
    run-to-run variance visible in CI logs.

    Args:
        exec_config: Execute configuration with 'command' and 'timeout'
        dist_dir: Distribution directory path
        input_dir: Input directory path
        output_dir: Output directory path
        iterations: Number of timed runs (must be >= 1)
        warmup: Number of untimed warmup runs (must be >= 0)
        label: Optional human-readable label used in the per-run log line

    Returns:
        Tuple of (success, median_seconds, all_timed_seconds). ``all_timed_seconds``
        is the list of timed-run wall-clock seconds in execution order. On failure
        it contains whatever timings were collected before the failed run.
    """
    if iterations < 1:
        raise ValueError(f"iterations must be >= 1 (got {iterations})")
    if warmup < 0:
        raise ValueError(f"warmup must be >= 0 (got {warmup})")

    prefix = f"[{label}] " if label else ""

    for i in range(warmup):
        _wipe(output_dir)
        ok, elapsed = execute_astgen(exec_config, dist_dir, input_dir, output_dir)
        print(
            f"{prefix}warmup {i + 1}/{warmup}: {elapsed:.3f}s",
            file=sys.stderr,
        )
        if not ok:
            return False, 0.0, []

    times: list[float] = []
    for i in range(iterations):
        _wipe(output_dir)
        ok, elapsed = execute_astgen(exec_config, dist_dir, input_dir, output_dir)
        print(
            f"{prefix}run {i + 1}/{iterations}: {elapsed:.3f}s",
            file=sys.stderr,
        )
        if not ok:
            return False, statistics.median(times) if times else 0.0, times
        times.append(elapsed)

    median = statistics.median(times)
    samples = ", ".join(f"{t:.3f}" for t in times)
    print(
        f"{prefix}median {median:.3f}s (samples: {samples})",
        file=sys.stderr,
    )
    return True, median, times


def _wipe(path: Path) -> None:
    """Remove ``path`` if it exists. Used to give each run a clean output tree."""
    if path.exists():
        shutil.rmtree(path)
