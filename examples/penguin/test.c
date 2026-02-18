#include <stdio.h>

typedef enum{
    ISFLOAT,
    ISINT,
    ISUNIT
} __Bob_tag;

typedef union {
    float f;
    int i;
    // unit ()
} Bob_variant;


typedef struct {
    __Bob_tag tag;
    Bob_variant variant;
} Bob;


int main() {
    auto bob = (Bob) {
    .tag = -1,
    };

    switch (bob.tag) {
        case ISFLOAT:
            printf("jemama");
            break;
        case ISINT:
            printf("jepapa");
            break;
        case ISUNIT:
            printf("kanker C");
            break;

    }
}