let best = 0, bestSteps = 0;
for (let n = 1; n < 1500000; n++) {
  let x = n, steps = 0;
  while (x !== 1) {
    if (x % 2 === 0) x = x / 2; else x = 3 * x + 1;
    steps++;
  }
  if (steps > bestSteps) { bestSteps = steps; best = n; }
}
console.log(best);
