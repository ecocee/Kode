#!/bin/bash
# Compare Kode with Python on the same recursive Fibonacci.
# Run from the project root after `cargo build --workspace --release`.
set -e
KODE=./target/release/kode
$KODE build benchmarks/fib/fib.kode --release -o /tmp/fib_kode
TIMEFORMAT='  %R seconds'
echo "Kode:";   time /tmp/fib_kode
echo "Python:"; time python3 benchmarks/fib/fib.py
