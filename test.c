// _________________ blocks as return values in functions ____________

fn block() {
    {
    a := 3;
    a
    }
}

void block() {
    int __return;
    int __block0;
    {
        int a = 3;
        __block0 = a;
    }
    __return = __block0;
    return __return;
}

fn block() {
    {
    a := 3;
    a
    }

    {
    b := 4;
    b
    }
}

void block() {
    int __return;
    int __block0;
    {
        int a = 3;
        __block0 = a;
    }
    int __block1;
        {
            int b = 4;
            __block1 = a;
        }
    __return = __block1;
    return __return;
}

// __________________ blocks as assignment values ___________________

fn f() {
    a :=
        { b := 3;
        b }
}


void f() {
    int a;

    // any block-expression gets a unique name
    int __block0_a;
    {
        int b = 3;
        __block0_a = b
    }

    a = __block0_a;
}

// ________________ tuple naming __________________-

fn length_squared(point: (int,int)) -> int {
    let (x,y) = point;

    return x * x + y * y;
}

struct __tuple_int_int;

struct __tuple_int_int {
    int _0, _1;
}

int length_squared(__tuple_int_int point) {
    let temp = point;
    int x = temp._0;
    int y = temp._1;
    return x * x + y * y;
}

