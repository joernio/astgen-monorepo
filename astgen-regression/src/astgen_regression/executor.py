"""Astgen execution functionality."""

from pathlib import Path
import subprocess
import sys
import time


def render_command(
    exec_config: dict,
    dist_dir: Path,
    input_dir: Path,
    output_dir: Path
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
        dist_dir=str(dist_dir),
        input_dir=str(input_dir),
        output_dir=str(output_dir)
    )


def execute_astgen(
    exec_config: dict,
    dist_dir: Path,
    input_dir: Path,
    output_dir: Path
) -> tuple[bool, float]:
    """Execute astgen binary and measure elapsed time.

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

    # Create output directory
    output_dir.mkdir(parents=True, exist_ok=True)

    t0 = time.monotonic()

    try:
        result = subprocess.run(
            cmd_str,
            shell=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout
        )
        elapsed = time.monotonic() - t0

        if result.returncode != 0:
            stderr = result.stderr.decode(errors='replace')[:500]
            print(
                f"WARNING: astgen exited {result.returncode}\n"
                f"  stderr: {stderr}",
                file=sys.stderr
            )
            return False, elapsed

        return True, elapsed

    except subprocess.TimeoutExpired:
        elapsed = time.monotonic() - t0
        print(
            f"WARNING: astgen timed out after {elapsed:.1f}s",
            file=sys.stderr
        )
        return False, elapsed

    except Exception as e:
        elapsed = time.monotonic() - t0
        print(f"WARNING: astgen execution failed: {e}", file=sys.stderr)
        return False, elapsed
