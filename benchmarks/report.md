# Kode benchmark results

Machine: macOS-27.2-arm64-arm-64bit-Mach-O, 10 CPU(s). Median of 3 runs.

## collatz

| Language | Median | Min | Max | Peak memory | Result |
|---|---|---|---|---|---|
| kode | 0.239 s | 0.239 s | 0.629 s | 1.8 MB | ok |
| c | 0.150 s | 0.148 s | 0.470 s | 1.6 MB | ok |
| cpp | 0.149 s | 0.147 s | 0.474 s | 1.6 MB | ok |
| go | 0.231 s | 0.225 s | 0.575 s | 4.3 MB | ok |
| javascript | 1.711 s | 1.710 s | 1.714 s | 47.4 MB | ok |
| python | 10.350 s | 10.228 s | 11.004 s | 14.2 MB | ok |

Ranking: cpp (0.149) < c (0.150) < go (0.231) < kode (0.239) < javascript (1.711) < python (10.350)

## fib

| Language | Median | Min | Max | Peak memory | Result |
|---|---|---|---|---|---|
| kode | 0.096 s | 0.096 s | 0.496 s | 1.8 MB | ok |
| c | 0.059 s | 0.059 s | 0.368 s | 1.6 MB | ok |
| cpp | 0.062 s | 0.058 s | 0.535 s | 1.6 MB | ok |
| go | 0.091 s | 0.091 s | 0.445 s | 4.2 MB | ok |
| javascript | 0.199 s | 0.195 s | 0.202 s | 46.8 MB | ok |
| python | 2.024 s | 2.022 s | 2.107 s | 14.2 MB | ok |

Ranking: c (0.059) < cpp (0.062) < go (0.091) < kode (0.096) < javascript (0.199) < python (2.024)

## loop_mod

| Language | Median | Min | Max | Peak memory | Result |
|---|---|---|---|---|---|
| kode | 0.037 s | 0.031 s | 0.588 s | 1.8 MB | ok |
| c | 0.022 s | 0.017 s | 0.326 s | 1.6 MB | ok |
| cpp | 0.017 s | 0.016 s | 0.319 s | 1.6 MB | ok |
| go | 0.036 s | 0.031 s | 0.633 s | 4.2 MB | ok |
| javascript | 0.049 s | 0.049 s | 0.051 s | 47.4 MB | ok |
| python | 2.378 s | 2.139 s | 2.731 s | 14.1 MB | ok |

Ranking: cpp (0.017) < c (0.022) < go (0.036) < kode (0.037) < javascript (0.049) < python (2.378)

## mandel

| Language | Median | Min | Max | Peak memory | Result |
|---|---|---|---|---|---|
| kode | 0.027 s | 0.022 s | 0.337 s | 1.8 MB | ok |
| c | 0.025 s | 0.019 s | 0.332 s | 1.6 MB | ok |
| cpp | 0.025 s | 0.019 s | 0.499 s | 1.6 MB | ok |
| go | 0.024 s | 0.019 s | 0.362 s | 4.4 MB | ok |
| javascript | 0.037 s | 0.032 s | 0.041 s | 48.6 MB | ok |
| python | 0.669 s | 0.646 s | 0.759 s | 14.2 MB | ok |

Ranking: go (0.024) < c (0.025) < cpp (0.025) < kode (0.027) < javascript (0.037) < python (0.669)

## sieve

| Language | Median | Min | Max | Peak memory | Result |
|---|---|---|---|---|---|
| kode | 0.072 s | 0.064 s | 0.568 s | 80.2 MB | ok |
| c | 0.022 s | 0.019 s | 0.331 s | 11.2 MB | ok |
| cpp | 0.022 s | 0.017 s | 0.392 s | 11.2 MB | ok |
| go | 0.029 s | 0.025 s | 0.369 s | 45.8 MB | ok |
| javascript | 0.152 s | 0.150 s | 0.179 s | 354.9 MB | ok |
| python | 1.435 s | 1.330 s | 1.503 s | 92.6 MB | ok |

Ranking: c (0.022) < cpp (0.022) < go (0.029) < kode (0.072) < javascript (0.152) < python (1.435)

