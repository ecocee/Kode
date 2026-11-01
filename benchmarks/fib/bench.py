
#!/usr/bin/env python3

import re
import statistics
import subprocess
import sys
import time
from pathlib import Path

# --------------------------------------------------
# Configuration
# --------------------------------------------------

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]

KODE = ROOT / "kode"
KODE_SOURCE = HERE / "ackermann.kode"
PY_SOURCE = HERE / "ackermann.py"
GO_SOURCE = HERE / "ackermann.go"

KODE_BINARY = HERE / "ackermann_bench"
GO_BINARY = HERE / "ackermann_go"

RUNS = 15
WARMUPS = 3

# --------------------------------------------------
# Helpers
# --------------------------------------------------

def run_checked(command, **kwargs):
    return subprocess.run(
        list(map(str, command)),
        check=True,
        **kwargs,
    )


def compile_programs():
    print("\n========== BUILDING BENCHMARKS ==========\n")

    if not KODE.exists():
        sys.exit(f"KODE compiler not found: {KODE}")

    for source in [KODE_SOURCE, PY_SOURCE, GO_SOURCE]:
        if not source.exists():
            sys.exit(f"Missing source file: {source}")

    print("[1/4] Building KODE...")

    run_checked(
        [
            KODE,
            "build",
            KODE_SOURCE.name,
            "--release",
            "-o",
            KODE_BINARY.name,
        ],
        cwd=HERE,
    )

    print("\n[2/4] Building Go...")

    run_checked(
        [
            "go",
            "build",
            "-trimpath",
            "-o",
            GO_BINARY,
            GO_SOURCE,
        ],
        cwd=HERE,
    )

    print("\nBuild completed.")


# --------------------------------------------------
# Measurement
# --------------------------------------------------

def measure(command):
    # High-resolution wall clock timer.
    start = time.perf_counter_ns()

    result = subprocess.run(
        ["/usr/bin/time", "-l", *map(str, command)],
        cwd=HERE,
        capture_output=True,
        text=True,
    )

    end = time.perf_counter_ns()

    if result.returncode != 0:
        raise RuntimeError(
            f"Execution failed:\n{result.stderr}"
        )

    elapsed = (end - start) / 1_000_000_000

    memory = re.search(
        r"(\d+)\s+maximum resident set size",
        result.stderr,
    )

    cpu = re.search(
        r"([\d.]+)\s+user\s+([\d.]+)\s+sys",
        result.stderr,
    )

    if not memory or not cpu:
        raise RuntimeError(
            "Could not parse macOS resource measurements:\n"
            + result.stderr
        )

    user, system = map(float, cpu.groups())

    return {
        "output": result.stdout.strip(),
        "wall": elapsed,
        "user": user,
        "system": system,
        "ram": int(memory.group(1)) / (1024 * 1024),
    }


# --------------------------------------------------
# Benchmark
# --------------------------------------------------

def benchmark(name, command):
    print(f"\nTesting {name}...")

    for _ in range(WARMUPS):
        measure(command)

    samples = []

    for i in range(RUNS):
        sample = measure(command)
        samples.append(sample)

        print(
            f"  Run {i + 1:02}/{RUNS}: "
            f"{sample['wall']:.6f}s | "
            f"{sample['ram']:.2f} MB"
        )

    outputs = {sample["output"] for sample in samples}

    if len(outputs) != 1:
        raise RuntimeError(
            f"{name} produced inconsistent outputs."
        )

    return {
        "output": samples[0]["output"],
        "wall": statistics.median(
            s["wall"] for s in samples
        ),
        "user": statistics.median(
            s["user"] for s in samples
        ),
        "system": statistics.median(
            s["system"] for s in samples
        ),
        "ram": statistics.median(
            s["ram"] for s in samples
        ),
    }


# --------------------------------------------------
# Main
# --------------------------------------------------

def main():
    compile_programs()

    programs = {
        "KODE": [KODE_BINARY],
        "Go": [GO_BINARY],
        "Python": [sys.executable, PY_SOURCE],
    }

    print("\n========== RUNNING BENCHMARKS ==========")
    print(f"Warm-ups: {WARMUPS}")
    print(f"Measured runs: {RUNS}")

    results = {}

    for name, command in programs.items():
        results[name] = benchmark(name, command)

    # --------------------------------------------------
    # Correctness
    # --------------------------------------------------

    outputs = {
        result["output"]
        for result in results.values()
    }

    if len(outputs) != 1:
        sys.exit(
            "\nFAILED: Programs produced different outputs: "
            + str({
                name: result["output"]
                for name, result in results.items()
            })
        )

    # --------------------------------------------------
    # Report
    # --------------------------------------------------

    print("\n========== FINAL RESULTS ==========\n")

    names = list(results.keys())

    print(
        f"{'METRIC':<22}"
        + "".join(f"{name:>16}" for name in names)
    )

    print("-" * (22 + 16 * len(names)))

    metrics = [
        ("Wall time", "wall", "s"),
        ("User CPU time", "user", "s"),
        ("System CPU time", "system", "s"),
        ("Peak RAM", "ram", "MB"),
    ]

    for label, key, unit in metrics:
        print(
            f"{label:<22}"
            + "".join(
                f"{results[name][key]:>12.4f} {unit:>3}"
                for name in names
            )
        )

    print("\n========== PERFORMANCE RATIOS ==========\n")

    for baseline in ["Go", "Python"]:
        ratio = (
            results[baseline]["wall"]
            / results["KODE"]["wall"]
        )

        print(
            f"KODE vs {baseline}: "
            f"{ratio:.2f}x "
            f"({'faster' if ratio > 1 else 'slower'})"
        )

    print("\n========== CORRECTNESS ==========\n")
    print("Output:", results["KODE"]["output"])
    print("All outputs match: PASSED")
    print("\nBenchmark completed.")


if __name__ == "__main__":
    main()
