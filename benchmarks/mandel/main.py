count = 0
for py in range(300):
    for px in range(300):
        x0 = px / 300.0 * 3.5 - 2.5
        y0 = py / 300.0 * 2.0 - 1.0
        x = 0.0
        y = 0.0
        i = 0
        while x * x + y * y <= 4.0 and i < 300:
            xt = x * x - y * y + x0
            y = 2.0 * x * y + y0
            x = xt
            i += 1
        count += i
print(count)
