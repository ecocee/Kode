let total = 0;
for (let i = 0; i < 50000000; i++) total += (i * 7 + 3) % 1000;
console.log(total);
