let count = 0;
for (let py = 0; py < 300; py++)
  for (let px = 0; px < 300; px++) {
    const x0 = px / 300.0 * 3.5 - 2.5;
    const y0 = py / 300.0 * 2.0 - 1.0;
    let x = 0.0, y = 0.0, i = 0;
    while (x * x + y * y <= 4.0 && i < 300) {
      const xt = x * x - y * y + x0;
      y = 2.0 * x * y + y0;
      x = xt;
      i++;
    }
    count += i;
  }
console.log(count);
