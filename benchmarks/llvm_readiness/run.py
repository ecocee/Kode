#!/usr/bin/env python3
"""Compare the Cranelift and LLVM backends on the readiness benchmark set.

Builds every case with `kode build --release --backend=...`, checks that both
backends print the same thing, then measures wall clock with hyperfine (median
of 20 runs after 2 warmups) and reports the LLVM/Cranelift ratio.

Usage: python3 benchmarks/llvm_readiness/run.py [--runs N] [--c]
Writes target/b7-bench.json and prints a Markdown table.
"""

import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
KODE = ROOT / "target/debug/kode"
OUT = Path("target/b7-bench")
HERE = Path(__file__).resolve().parent

CASES = [
    # name, source, program args, has a C version
    ("fib", ROOT / "benchmarks/fib/main.kode", [], True),
    ("loop_mod", ROOT / "benchmarks/loop_mod/main.kode", [], True),
    ("mandel", ROOT / "benchmarks/mandel/main.kode", [], True),
    ("sieve", ROOT / "benchmarks/sieve/main.kode", [], True),
    ("collatz", ROOT / "benchmarks/collatz/main.kode", [], True),
    ("lists", HERE / "lists.kode", [], False),
    ("packed_bools", HERE / "packed.kode", [], False),
    ("maps", HERE / "maps.kode", [], False),
    ("json", HERE / "json.kode", [], False),
    ("strings", HERE / "strings.kode", [], False),
    ("objects", HERE / "objects.kode", [], False),
    ("tasks_channels", HERE / "tasks.kode", [], False),
    ("app_wordcount", ROOT / "examples/wordcount.kode", ["target/corpus.txt"], False),
]


def corpus() -> None:
    """A few MB of Markdown, for the wordcount application case."""
    path = ROOT / "target/corpus.txt"
    if path.exists() and path.stat().st_size > 1_000_000:
        return
    parts = []
    for p in sorted((ROOT / "docs").rglob("*.md")) + sorted(ROOT.glob("*.md")):
        parts.append(p.read_text())
    text = "\n".join(parts)
    path.write_text(text * (3_000_000 // max(1, len(text)) + 1))


def build(name: str, src: Path, backend: str) -> Path:
    exe = OUT / f"{name}-{backend}"
    args = [str(KODE), "build", str(src), "-o", str(exe), "--release", f"--backend={backend}"]
    r = subprocess.run(args, capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(f"{name}: {backend} build failed:\n{r.stderr}")
    return exe


def run(exe: Path, args: list[str]) -> str:
    r = subprocess.run(
        [str(exe)] + args,
        capture_output=True,
        text=True,
        env=dict(os.environ, KODE_LEAK_CHECK="1"),
    )
    if r.returncode != 0 or r.stderr.strip():
        sys.exit(f"{exe.name}: exit {r.returncode}, stderr:\n{r.stderr}")
    return r.stdout


def measure(exe: Path, args: list[str], runs: int) -> float:
    js = OUT / f"{exe.name}.json"
    r = subprocess.run(
        [
            "hyperfine",
            "--warmup",
            "2",
            "--runs",
            str(runs),
            "--export-json",
            str(js),
            "--style",
            "none",
            " ".join([str(exe)] + args),
        ],
        capture_output=True,
        text=True,
    )
    if r.returncode != 0:
        sys.exit(f"hyperfine failed for {exe.name}:\n{r.stderr}")
    data = json.loads(js.read_text())
    return data["results"][0]["summary"]["time_wall_clock"]["median"] * 1000.0


def main() -> None:
    runs = 20
    want_c = "--c" in sys.argv
    if "--runs" in sys.argv:
        runs = int(sys.argv[sys.argv.index("--runs") + 1])
    OUT.mkdir(parents=True, exist_ok=True)
    corpus()

    rows = []
    for name, src, args, has_c in CASES:
        cr = build(name, src, "cranelift")
        ll = build(name, src, "llvm")
        a = run(cr, args)
        b = run(ll, args)
        if a != b:
            sys.exit(f"{name}: backends disagree\ncranelift:\n{a}\nllvm:\n{b}")
        c_ms = None
        if want_c and has_c:
            c_exe = OUT / f"{name}-c"
            cc = subprocess.run(
                ["cc", "-O2", "-o", str(c_exe), str(src.parent / "main.c")],
                capture_output=True,
                text=True,
            )
            if cc.returncode == 0:
                if run(c_exe, []) != a:
                    sys.exit(f"{name}: C output differs")
                c_ms = measure(c_exe, [], runs)
        rows.append(
            {
                "case": name,
                "cranelift_ms": measure(cr, args, runs),
                "llvm_ms": measure(ll, args, runs),
                "c_ms": c_ms,
                "output": a.strip().splitlines()[-1] if a.strip() else "",
            }
        )
        print(f"{name}: done", flush=True)

    (ROOT / "target/b7-bench.json").write_text(json.dumps(rows, indent=2))
    print()
    print("| case | Cranelift ms | LLVM ms | LLVM/Cranelift | C ms | LLVM/C |")
    print("| --- | ---: | ---: | ---: | ---: | ---: |")
    for r in rows:
        c = f"{r['c_ms']:.1f}" if r["c_ms"] is not None else "—"
        ratio_c = (
            f"{r['llvm_ms'] / r['c_ms']:.2f}" if r["c_ms"] and r["c_ms"] > 0 else "—"
        )
        print(
            f"| {r['case']} | {r['cranelift_ms']:.1f} | {r['llvm_ms']:.1f} | "
            f"{r['llvm_ms'] / r['cranelift_ms']:.2f} | {c} | {ratio_c} |"
        )


if __name__ == "__main__":
    main()
