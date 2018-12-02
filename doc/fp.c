#include <stdio.h>

#define SHIFT_AMOUNT 8 // 2^8 = 256
#define SHIFT_MASK ((1 << SHIFT_AMOUNT) - 1) // 255 (all LSB set, all MSB clear)
#define SHIFT_MSB (1 << (SHIFT_AMOUNT - 1))

#define WHOLE(x) (x>>SHIFT_AMOUNT)
#define FRACT(x) ((long long int)(x & SHIFT_MASK) * 100000 / (1 << SHIFT_AMOUNT))

#define DUMP(name, x) printf("%-15s  %3d .. %8lld [%8d]  ", name, WHOLE(x), FRACT(x), x) ; pbits(x) ; printf("\n");

void
pbits(unsigned int v) {
    int n = sizeof(v)*8-1;
    for(int i = sizeof(v)*8-1; i >= 0; i--) {
        if(i%8 == 7 && i != n) {
            printf("_");
        }
        printf("%u", (v >> i) & 1);
    }
}

int
main() {

    for(int i = 0; i < 256; i++) {
        for(int j = 0; j < 256; j++) {
            unsigned int a = i; // 8 bits
            unsigned int b = j; // 8 bits
            //unsigned int d = 256; // 8 bits

            unsigned int p  = ((unsigned int)a * (unsigned int)b) ;
            unsigned int c  = ((unsigned int)a * (unsigned int)b) + SHIFT_MSB;
            //unsigned int c0 = ((unsigned int)a * (unsigned int)b) ;
            //printf("c0 %d .. %lld [%d]\n", WHOLE(c0), FRACT(c0), c0);
            //printf("%d .. %lld [%d]\n", WHOLE(d), FRACT(d), d);

            unsigned int t = (c >> SHIFT_AMOUNT);
            //unsigned int t0 = (c0 >> SHIFT_AMOUNT);
            //printf("t0 %d .. %lld [%d]\n", WHOLE(t0), FRACT(t0), t0);
            unsigned int r = (t + c) >> SHIFT_AMOUNT;
            unsigned int r2 = ((c >> SHIFT_AMOUNT) + c) >> SHIFT_AMOUNT;
            //unsigned int r0 = t0 + c0;
            //printf("r0 %d .. %lld [%d]\n", WHOLE(r0), FRACT(r0), r0);
            //unsigned int v = (t >> SHIFT_AMOUNT) + t;
            //unsigned int x = t >> SHIFT_AMOUNT;
            //unsigned int y = t;
            if(r2 - t == 1) {
                printf("\n:::: %d %d\n", i, j);
                //printf("%d .. %lld [%d]\n", WHOLE(a), FRACT(a), a);
                //printf("%d .. %lld [%d]\n", WHOLE(b), FRACT(b), b);
                //printf("c  %d .. %lld [%d]\n", WHOLE(c), FRACT(c), c);
                //DUMP("a", a);
                //DUMP("b", b);
                printf("------------- %d\n", (p >> 7) & 1);
                DUMP("p", p);
                DUMP("c", c);
                DUMP(" (c>>8)", (c>>SHIFT_AMOUNT));
                DUMP(" (c>>8)+c", ((c>>SHIFT_AMOUNT) + c));
                DUMP("((c>>8)+c)>>8", (((c>>SHIFT_AMOUNT) + c) >> SHIFT_AMOUNT));
                DUMP("r", r);
                //DUMP("v", v);
                if(((p >>7) & 1) == 1) {
                    exit(0);
                }
            }
        }
    }
    return 0;
}
