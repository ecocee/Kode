#include <stdio.h>
int main(void) {
    long long best = 0, best_steps = 0;
    for (long long n = 1; n < 1500000; n++) {
        long long x = n, steps = 0;
        while (x != 1) {
            if (x % 2 == 0) x = x / 2; else x = 3 * x + 1;
            steps++;
        }
        if (steps > best_steps) { best_steps = steps; best = n; }
    }
    printf("%lld\n", best);
    return 0;
}
