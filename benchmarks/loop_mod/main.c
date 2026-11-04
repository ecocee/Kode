#include <stdio.h>
int main(void) {
    long long total = 0;
    for (long long i = 0; i < 50000000; i++) total += (i * 7 + 3) % 1000;
    printf("%lld\n", total);
    return 0;
}
