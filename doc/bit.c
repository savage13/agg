
#include <stdio.h>

int main() {

    unsigned int flags = 0;
    for( int i = 0; i < 4; i++) {
        flags = i;
        printf("%d : %d : (flags & 1) == 0\n", flags, (flags & 1) );
    }
    
    return 0;
}
