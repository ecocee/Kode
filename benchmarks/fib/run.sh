
#!/usr/bin/env bash
set -euo pipefail

echo "Building KODE..."
./kode build ackermann.kode --release -o ackermann

TIMEFORMAT='  %R seconds'

echo ""
echo "========== KODE vs Python =========="

echo "KODE:"
time ./ackermann

echo ""
echo "Python:"
time python3 ackermann.py
