total = 0
for i in range(50000000):
    total += (i * 7 + 3) % 1000
print(total)
