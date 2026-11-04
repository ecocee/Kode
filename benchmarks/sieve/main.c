#include <stdio.h>
#include <stdlib.h>
int main(void) {
    long long n = 10000000;
    char *f = (char *)malloc(n + 1);
    for (long long i = 0; i <= n; i++) f[i] = 1;
    f[0] = 0; f[1] = 0;
    for (long long i = 2; i * i <= n; i++)
        if (f[i]) for (long long j = i * i; j <= n; j += i) f[j] = 0;
    long long count = 0;
    for (long long k = 0; k <= n; k++) if (f[k]) count++;
    printf("%lld\n", count);
    return 0;
}
