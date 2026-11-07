# HTTP benchmark report
Generated: 2026-11-07T02:36:05.186426 (container, 1 vCPU / 1 GiB)
Runner: `benchmarks/http/load.go` (purpose-written keep-alive client) with `-n/-c`; see `benchmarks/http/measure.sh`.
All servers built for release where possible; Kode server built with `kode build <file> --release`.

## Results

| language | endpoint | connections | requests | actual | rps | p50 (ms) | p99 (ms) | errors | RSS (KB) | peak RSS (KB) |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| kode | hello | 100 | 1000 | 1000 | 40757 | 0.06 | 14.38 | 0 | 1572 | 1572 |
| kode | hello | 100 | 100000 | 100000 | 23419 | 0.06 | 0.13 | 0 | 1572 | 1572 |
| kode | hello | 1000 | 1000 | 1000 | 640 | 209.48 | 438.34 | 0 | 1572 | 1572 |
| kode | hello | 1000 | 100000 | 100000 | 3576 | 0.06 | 66.60 | 0 | 1572 | 1572 |
| kode | json | 100 | 1000 | 1000 | 32586 | 0.06 | 18.04 | 0 | 1600 | 1600 |
| kode | json | 100 | 100000 | 100000 | 21962 | 0.06 | 0.13 | 0 | 1632 | 1632 |
| kode | json | 1000 | 1000 | 1000 | 361 | 210.56 | 1718.63 | 0 | 1632 | 1632 |
| kode | json | 1000 | 100000 | 100000 | 3572 | 0.06 | 68.92 | 0 | 1632 | 1632 |
| go | hello | 100 | 1000 | 1000 | 65801 | 0.17 | 10.38 | 0 | 14980 | 14980 |
| go | hello | 100 | 100000 | 100000 | 36594 | 0.23 | 85.61 | 0 | 15808 | 17028 |
| go | hello | 1000 | 1000 | 1000 | 5436 | 3.12 | 83.75 | 0 | 28536 | 28536 |
| go | hello | 1000 | 100000 | 100000 | 31270 | 3.96 | 191.88 | 0 | 35408 | 51044 |
| go | json | 100 | 1000 | 1000 | 13055 | 0.12 | 68.52 | 0 | 35408 | 51044 |
| go | json | 100 | 100000 | 100000 | 28712 | 0.25 | 84.88 | 0 | 20384 | 51044 |
| go | json | 1000 | 1000 | 1000 | 5391 | 10.35 | 102.86 | 0 | 32368 | 51044 |
| go | json | 1000 | 100000 | 100000 | 28697 | 3.59 | 108.10 | 0 | 29712 | 51044 |
| node | hello | 100 | 1000 | 1000 | 15674 | 4.06 | 17.37 | 0 | 59268 | 59268 |
| node | hello | 100 | 100000 | 100000 | 27181 | 1.39 | 62.77 | 0 | 74856 | 74856 |
| node | hello | 1000 | 1000 | 1000 | 5309 | 15.50 | 96.65 | 0 | 81580 | 81604 |
| node | hello | 1000 | 100000 | 100000 | 26905 | 18.41 | 79.88 | 0 | 105804 | 105804 |
| node | json | 100 | 1000 | 1000 | 16385 | 1.47 | 43.77 | 0 | 105860 | 105860 |
| node | json | 100 | 100000 | 100000 | 26496 | 1.59 | 57.02 | 0 | 106448 | 106448 |
| node | json | 1000 | 1000 | 1000 | 5154 | 9.24 | 105.54 | 0 | 107288 | 107288 |
| node | json | 1000 | 100000 | 100000 | 25667 | 19.70 | 79.36 | 0 | 113732 | 113732 |
| python | hello | 100 | 1000 | 940 | 30 | 42.65 | 15041.72 | 6 | 17916 | 17916 |
| python | hello | 100 | 100000 | 74000 | 1171 | 43.78 | 51.31 | 26 | 20756 | 21056 |
| python | hello | 1000 | 1000 | 184 | 6 | 7.47 | 27453.64 | 816 | 20832 | 21056 |
| python | hello | 1000 | 100000 | 14600 | 440 | 42.70 | 53.97 | 854 | 21684 | 23232 |
| python | json | 100 | 1000 | 800 | 26 | 43.05 | 14915.01 | 20 | 21676 | 23232 |
| python | json | 100 | 100000 | 50000 | 806 | 44.12 | 47.62 | 50 | 21760 | 23232 |
| python | json | 1000 | 1000 | 207 | 6 | 207.01 | 28158.72 | 793 | 21764 | 23232 |
| python | json | 1000 | 100000 | 18600 | 562 | 43.09 | 62.09 | 814 | 22480 | 24624 |

## Notes

- Python uses `threading` + `select` in the benchmark server; high p99 and errors under load are expected for that model.
- Kode HTTP runtime uses `available_parallelism().max(4)` workers (ADR context) and a global `ROUTES` lock on dispatch; performance is constrained by that design.
- The custom Go load client prints accurate counters (we fixed flag parsing) unlike `hey` which mis-reported Kode totals under concurrency.
