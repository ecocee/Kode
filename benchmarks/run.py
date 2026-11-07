#!/usr/bin/env python3
"""Measure the benchmarks and write benchmarks/report.md.

Wall clock comes from `hyperfine` (median/min/max and standard deviation over
its runs) when it is installed; otherwise the best of RUNS wall-clock runs is
used and the report says so. Peak RSS is each child's own high-water mark, read
from the platform `time` tool, so one program cannot hide inside another's
number.

    RUNS=3 ./run.sh            # or: python3 benchmarks/run.py

Only tools that are installed are measured; a missing one is left out with a
note in the report.
"""

from __future__ import annotations

import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
KODE = ROOT / "target" / "debug" / "kode"
BUILD = HERE / "build"
RUNS = int(os.environ.get("RUNS", "3"))
PROGRAMS = ["fib", "loop_mod", "sieve", "mandel", "collatz"]
TIME = shutil.which("time") or "/usr/bin/time"
LANGUAGES = ["Kode", "C", "Go", "Node", "Python"]


def best_ms(cmd: list[str], cwd: Path) -> float | None:
    best = None
    for _ in range(RUNS):
        start = time.perf_counter()
        try:
            r = subprocess.run(cmd, cwd=cwd, capture_output=True, timeout=180)
        except (subprocess.TimeoutExpired, OSError):
            return None
        if r.returncode != 0:
            return None
        ms = (time.perf_counter() - start) * 1000.0
        best = ms if best is None else min(best, ms)
    return best


def hyperfine_ms(cmd: list[str], cwd: Path) -> dict[str, float] | None:
    """median/min/max/standard deviation in milliseconds, and peak RSS in KB."""
    if not shutil.which("hyperfine"):
        return None
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as f:
        out = f.name
    try:
        r = subprocess.run(
            [
                "hyperfine", "--warmup", "2", "--min-runs", "5", "--max-runs", "20",
                "--export-json", out, "-N", " ".join(cmd),
            ],
            cwd=cwd, capture_output=True, text=True, timeout=600,
        )
        if r.returncode != 0:
            return None
        data = json.loads(Path(out).read_text())["results"][0]
        # hyperfine 1.x reports the numbers flat; 2.x nests them under
        # "summary", per metric, in seconds and bytes.
        if "summary" in data:
            wall = data["summary"]["time_wall_clock"]
            mem = data["summary"].get("memory_peak_resident", {})
            stats = {
                "median": wall["median"] * 1000,
                "min": wall["min"] * 1000,
                "max": wall["max"] * 1000,
                "sd": wall["stddev"] * 1000,
            }
            if "median" in mem:
                stats["rss_kb"] = mem["median"] / 1024
            return stats
        return {
            "median": data["median"] * 1000,
            "min": data["min"] * 1000,
            "max": data["max"] * 1000,
            "sd": data["stddev"] * 1000,
        }
    except (OSError, ValueError, KeyError):
        return None
    finally:
        Path(out).unlink(missing_ok=True)


def peak_rss_kb(cmd: list[str], cwd: Path) -> int | None:
    """Peak RSS of one run, in kilobytes."""
    if not Path(TIME).exists():
        return None
    flag = "-l" if platform.system() == "Darwin" else "-v"
    try:
        r = subprocess.run([TIME, flag, *cmd], cwd=cwd, capture_output=True, text=True, timeout=180)
    except (subprocess.TimeoutExpired, OSError):
        return None
    # macOS reports bytes, with the number before the label.
    m = re.search(r"(\d+)\s+maximum resident set size", r.stderr, re.IGNORECASE)
    if m:
        return int(m.group(1)) // 1024
    # Linux reports kilobytes, with the number after the label.
    m = re.search(r"Maximum resident set size \(kbytes\):\s*(\d+)", r.stderr, re.IGNORECASE)
    if m:
        return int(m.group(1))
    return None


def have(tool: str) -> bool:
    return shutil.which(tool) is not None


def build(program: str, sources: dict[str, Path]) -> dict[str, list[str]]:
    """Build every available variant; return name -> command line."""
    out = BUILD / program
    out.mkdir(parents=True, exist_ok=True)
    runs: dict[str, list[str]] = {}

    exe = out / "program"
    r = subprocess.run(
        [str(KODE), "build", str(sources["kode"]), "-o", str(exe), "--release"],
        capture_output=True, text=True,
    )
    if r.returncode == 0:
        runs["Kode"] = [str(exe)]

    if have("cc"):
        exe_c = out / "c"
        if subprocess.run(["cc", "-O2", "-o", str(exe_c), str(sources["c"])], capture_output=True).returncode == 0:
            runs["C"] = [str(exe_c)]
    if have("go"):
        exe_go = out / "go"
        if subprocess.run(["go", "build", "-o", str(exe_go), str(sources["go"])], capture_output=True).returncode == 0:
            runs["Go"] = [str(exe_go)]
    if have("node"):
        runs["Node"] = ["node", str(sources["js"])]
    if have("python3"):
        runs["Python"] = ["python3", str(sources["py"])]
    return runs


def tool_versions() -> list[str]:
    def first(cmd: list[str]) -> str:
        try:
            r = subprocess.run(cmd, capture_output=True, text=True, timeout=20)
            text = (r.stdout or r.stderr).strip().splitlines()
            return text[0] if text else "?"
        except (OSError, subprocess.TimeoutExpired):
            return "?"

    return [
        f"- compiler: `{KODE}` — {first([str(KODE), '--version'])}",
        f"- C: {first(['cc', '--version'])}",
        f"- Go: {first(['go', 'version'])}",
        f"- Node: {first(['node', '--version'])}",
        f"- Python: {platform.python_version()} (`{shutil.which('python3')}`)",
        f"- hyperfine: {first(['hyperfine', '--version']) if have('hyperfine') else 'not installed'}",
    ]


def main() -> int:
    if not KODE.exists():
        print(f"build the compiler first: cargo build --workspace ({KODE} is missing)", file=sys.stderr)
        return 1
    BUILD.mkdir(exist_ok=True)

    rows = []          # one dict per program: {"program", "stats": {lang: {...}}, "rss": {lang: kb}}
    sizes = {}
    for program in PROGRAMS:
        d = HERE / program
        if not (d / "main.kode").exists():
            continue
        sources = {
            "kode": d / "main.kode",
            "c": d / "main.c",
            "go": d / "main.go",
            "js": d / "main.js",
            "py": d / "main.py",
        }
        runs = build(program, sources)
        row: dict[str, object] = {"program": program, "stats": {}, "rss": {}}
        for name, cmd in runs.items():
            stats = hyperfine_ms(cmd, d)
            if stats is None:
                ms = best_ms(cmd, d)
                if ms is not None:
                    stats = {"median": ms, "min": ms, "max": ms, "sd": 0.0, "best_only": True}
            if stats is not None:
                row["stats"][name] = stats
            row["rss"][name] = stats.pop("rss_kb", None) if stats else None
            if row["rss"][name] is None:
                row["rss"][name] = peak_rss_kb(cmd, d)
        if "Kode" in runs:
            exe = Path(runs["Kode"][0])
            if exe.exists():
                sizes[program] = exe.stat().st_size
        rows.append(row)
        print(f"measured {program}", file=sys.stderr)

    langs = [lang for lang in LANGUAGES if any(lang in r["stats"] for r in rows)]
    lines = [
        "# Benchmark report",
        "",
        "Generated by `benchmarks/run.sh`.",
        "",
        f"- machine: {platform.system()} {platform.release()} / {platform.machine()}",
        f"- cores: {os.cpu_count()}",
        f"- Rust: {platform.python_version() if False else subprocess.run(['rustc', '--version'], capture_output=True, text=True).stdout.strip()}",
    ]
    lines += tool_versions()
    lines += [
        "",
        "Wall clock is the median of `hyperfine` runs (2 warmups, 5 to 20 runs), with",
        "min, max and standard deviation; peak RSS is the child's own high-water mark",
        f"from `{'/usr/bin/time -l' if platform.system() == 'Darwin' else '/usr/bin/time -v'}`.",
        "Every binary is built for release: `kode build --release`, `cc -O2` and plain",
        "`go build`; Node and Python run their source.",
        "",
        "## Median wall clock (ms)",
        "",
        "| program | " + " | ".join(langs) + " | Kode/C |",
        "|---|" + "---:|" * len(langs) + "---:|",
    ]
    for row in rows:
        stats = row["stats"]
        cells = [f"{stats[l]['median']:.0f}" if l in stats else "—" for l in langs]
        ratio = "—"
        if "Kode" in stats and "C" in stats:
            ratio = f"{stats['Kode']['median'] / stats['C']['median']:.2f}×"
        lines.append(f"| {row['program']} | " + " | ".join(cells) + f" | {ratio} |")

    lines += ["", "## Full statistics and peak RSS", ""]
    for row in rows:
        lines += [
            f"### `{row['program']}`",
            "",
            "| runtime | median (ms) | min (ms) | max (ms) | std dev (ms) | peak RSS |",
            "|---|---:|---:|---:|---:|---:|",
        ]
        for lang in langs:
            s = row["stats"].get(lang)
            rss = row["rss"].get(lang)
            rss_txt = f"{rss} KB" if rss else "—"
            if not s:
                lines.append(f"| {lang} | — | — | — | — | {rss_txt} |")
                continue
            sd = "—" if s.get("best_only") else f"{s['sd']:.1f}"
            lines.append(
                f"| {lang} | {s['median']:.1f} | {s['min']:.1f} | {s['max']:.1f} | {sd} | {rss_txt} |"
            )
        lines.append("")

    if sizes:
        lines += ["## Binary size (`--release`, stripped)", ""]
        for program, size in sizes.items():
            lines.append(f"- `{program}`: {size / 1024:.0f} KB")
        lines.append("")

    lines += [
        "Ratios use the median against C `-O2` where both ran; a dash means the",
        "tool is not installed on this machine.",
        "",
    ]
    report = HERE / "report.md"
    report.write_text("\n".join(lines))
    print(f"wrote {report}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
