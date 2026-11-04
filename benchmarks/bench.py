#!/usr/bin/env python3
"""Kode benchmark harness.

    python3 benchmarks/bench.py                 # every benchmark, every language found
    python3 benchmarks/bench.py fib sieve       # only these benchmarks
    python3 benchmarks/bench.py --runs 5 --langs kode,c,go

Each benchmark is a folder with main.kode, main.c, main.go, main.js, main.py
(add your own!) and an `expected.txt` with the right answer. A language whose tool
is not installed is skipped. Memory is the real peak resident size of each run.
"""
import argparse, os, platform, shutil, statistics, subprocess, sys, tempfile, time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)


def find_kode():
    for c in [os.environ.get("KODE"), os.path.join(ROOT, "target/release/kode"),
              os.path.join(ROOT, "target/debug/kode"), shutil.which("kode")]:
        if c and os.path.exists(c):
            return c
    return None


KODE = find_kode()

# language -> (source file, tool to look for, build command or None, run command)
def langs(src, out):
    return {
        "kode":       ("main.kode", KODE, [KODE, "build", src, "-o", out, "--release"] if KODE else None, [out]),
        "c":          ("main.c", "gcc", ["gcc", "-O2", "-ffp-contract=off", "-o", out, src, "-lm"], [out]),
        "cpp":        ("main.c", "g++", ["g++", "-O2", "-ffp-contract=off", "-x", "c++", "-o", out, src, "-lm"], [out]),
        "go":         ("main.go", "go", ["go", "build", "-o", out, src], [out]),
        "javascript": ("main.js", "node", None, ["node", src]),
        "python":     ("main.py", "python3", None, ["python3", src]),
    }


def run_once(cmd, timeout):
    """Run a command; return (seconds, peak_rss_bytes, stdout, ok)."""
    with tempfile.TemporaryFile() as out:
        t0 = time.perf_counter()
        p = subprocess.Popen(cmd, stdout=out, stderr=subprocess.DEVNULL)
        peak = 0
        while True:
            if sys.platform.startswith("linux"):
                try:  # peak resident size of the running program itself
                    for line in open(f"/proc/{p.pid}/status"):
                        if line.startswith("VmHWM:"):
                            peak = max(peak, int(line.split()[1]) * 1024)
                except OSError:
                    pass
            done, status, ru = os.wait4(p.pid, os.WNOHANG)
            if done:
                break
            if time.perf_counter() - t0 > timeout:
                p.kill(); os.wait4(p.pid, 0)
                return None, 0, "", False
            time.sleep(0.002)
        secs = time.perf_counter() - t0
        out.seek(0)
        text = out.read().decode(errors="replace").strip()
    # Linux: sampled from /proc. macOS: the kernel's number (it includes a few MB from the harness).
    rss = peak or ru.ru_maxrss * (1 if platform.system() == "Darwin" else 1024)
    return secs, rss, text, os.WIFEXITED(status) and os.WEXITSTATUS(status) == 0


def fmt_bytes(n):
    return f"{n/1048576:.1f} MB" if n else "n/a"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("names", nargs="*")
    ap.add_argument("--runs", type=int, default=3)
    ap.add_argument("--langs", default="")
    ap.add_argument("--timeout", type=float, default=120)
    a = ap.parse_args()
    names = a.names or sorted(d for d in os.listdir(HERE) if os.path.isfile(os.path.join(HERE, d, "expected.txt")))
    wanted = [x for x in a.langs.split(",") if x]
    report = ["# Kode benchmark results", "", f"Machine: {platform.platform()}, {os.cpu_count()} CPU(s). Median of {a.runs} runs.", ""]
    for name in names:
        d = os.path.join(HERE, name)
        expected = open(os.path.join(d, "expected.txt")).read().strip()
        print(f"\n== {name} (expected {expected})")
        report += [f"## {name}", "", "| Language | Median | Min | Max | Peak memory | Result |", "|---|---|---|---|---|---|"]
        tmp = tempfile.mkdtemp(prefix="kodebench-")
        results = []
        for lang in ["kode", "c", "cpp", "go", "javascript", "python"]:
            if wanted and lang not in wanted:
                continue
            src_name, tool, build, run = langs("", "")[lang]
            src = os.path.join(d, src_name)
            if not os.path.exists(src):
                continue
            out = os.path.join(tmp, lang)
            _, tool, build, run = langs(src, out)[lang]
            if not tool or not shutil.which(tool) and not os.path.exists(str(tool)):
                print(f"{lang:11} SKIPPED (not installed)"); report.append(f"| {lang} | skipped | | | | not installed |"); continue
            if build:
                b = subprocess.run(build, capture_output=True, text=True)
                if b.returncode != 0:
                    print(f"{lang:11} BUILD FAILED: {b.stderr.strip()[:200]}"); report.append(f"| {lang} | build failed | | | | |"); continue
            times, rss, ok = [], 0, True
            for _ in range(a.runs):
                secs, r, text, good = run_once(run, a.timeout)
                if secs is None:
                    ok = False; text = "timeout"; break
                if not good or text != expected:
                    ok = False; text = text[:40]; break
                times.append(secs); rss = max(rss, r)
            if not ok:
                print(f"{lang:11} WRONG ({text})"); report.append(f"| {lang} | wrong | | | | {text} |"); continue
            med = statistics.median(times)
            results.append((med, lang))
            print(f"{lang:11} median {med:7.3f}s  min {min(times):7.3f}s  max {max(times):7.3f}s  memory {fmt_bytes(rss)}")
            report.append(f"| {lang} | {med:.3f} s | {min(times):.3f} s | {max(times):.3f} s | {fmt_bytes(rss)} | ok |")
        shutil.rmtree(tmp, ignore_errors=True)
        report.append("")
        if results:
            results.sort()
            report.append("Ranking: " + " < ".join(f"{l} ({t:.3f})" for t, l in results))
            report.append("")
    with open(os.path.join(HERE, "report.md"), "w") as f:
        f.write("\n".join(report) + "\n")
    print("\nwrote benchmarks/report.md")


main()
