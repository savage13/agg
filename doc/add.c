

#include <stdio.h>
int
main() {
    unsigned char a = 200;
    unsigned char b = 200;
    unsigned char c = 0;

    c = a + b;
    printf("%lu bytes\n", sizeof(a));
    printf("%3u %3u %3u\n", a, b, c);
    return 0;
}
