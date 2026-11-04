#include <stdio.h>
int main(void) {
    long long count = 0;
    for (int py = 0; py < 300; py++)
        for (int px = 0; px < 300; px++) {
            double x0 = (double)px / 300.0 * 3.5 - 2.5;
            double y0 = (double)py / 300.0 * 2.0 - 1.0;
            double x = 0.0, y = 0.0;
            int i = 0;
            while (x * x + y * y <= 4.0 && i < 300) {
                double xt = x * x - y * y + x0;
                y = 2.0 * x * y + y0;
                x = xt;
                i++;
            }
            count += i;
        }
    printf("%lld\n", count);
    return 0;
}
