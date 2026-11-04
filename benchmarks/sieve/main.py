n = 10000000
f = []
for i in range(n + 1):
    f.append(True)
f[0] = False
f[1] = False
i = 2
while i * i <= n:
    if f[i]:
        j = i * i
        while j <= n:
            f[j] = False
            j += i
    i += 1
count = 0
for k in range(n + 1):
    if f[k]:
        count += 1
print(count)
