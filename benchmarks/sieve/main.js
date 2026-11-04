const n = 10000000;
const f = [];
for (let i = 0; i <= n; i++) f.push(true);
f[0] = false; f[1] = false;
for (let i = 2; i * i <= n; i++) {
  if (f[i]) for (let j = i * i; j <= n; j += i) f[j] = false;
}
let count = 0;
for (let k = 0; k <= n; k++) if (f[k]) count++;
console.log(count);
