
import sys

sys.setrecursionlimit(10000)

def ack(m, n):
    if m == 0:
        return n + 1
    if n == 0:
        return ack(m - 1, 1)
    return ack(m - 1, ack(m, n - 1))

total = 0

for i in range(100):
    total += ack(3, 8)

print(total)
