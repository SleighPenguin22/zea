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

//

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

struct __tuple2_int_int;

struct __tuple2_int_int {
    int _0; int _1;
}

int length_squared(__tuple2_int_int point) {
    let __unpack = point;
    int x = __unpack._0;
    int y = __unpack._1;
    return x * x + y * y;
}

//

fn f(tup: ((int,int),int,bool)) {
    let ((x,y),z,w) = tup;
}

struct __tuple2_int_int;
struct __tuple2_int_int {int _0; int _1;}

struct __tuple3_tuple2_int_int_int_bool
struct __tuple3_tuple2_int_int_int_bool {__tuple2_int_int _0; int _1; bool _2}

void f(__tuple3_tuple2_int_int_int_bool tup) {
    let __unpack = tup;

    let __unpack_xy = __unpack._0;
    let x = __unpack_xy._0;
    let y = __unpack_xy._1;

    let z = __unpack._1;
    let w = __unpack._2;
}

//

fn sign(n: int) -> int {
    match {
        a < 0 => {
        inner := -1;
        inner
        },
        a > 0 => 1,
        else => {
            0
        }
    }
}

int sign(int n) {
    // implicit returns get assigned to __return
    int __return;

    // condition matches generate a unique variable
    int __match0;
    if (a<0) {

        // blocks generate a unique variable
        int __block0;
        {
            int inner = -1;
            // that gets assigned within a scope
            __block0 = inner;
        }

        // each branch assigns to the generated condmatch variable
        __match0 = __block0;
    } else if (a>0) {
        // if a condmatch arm is a simple expression (not a block or match)
        // it can be assigned directly to the arm
        __match0 = 1;
    } else {
        int __block1;
        {
            // the last (sometimes only) expression in a block
            // is assigned to the generated block-variable
            __block1 = 0;
        }
        __match0 = __block1;
    }
    __return = __match0;
    return __return;
}

// _________________ signatures of expression-lowering transformations ________

single-define-tuple
:: Unnamed Tuple
-> (
   field_descriptor := join stringified field in tuple by _
   C struct __tuplen_field_descriptor (field of tuples)
)

single-desugar-unpack-tuple
:: Sugared Initialisation ->  (
    C global declare tuple,
    C global define tuple,
    C init temp = tuple,
    for i, field in unpack (
        C assign field = tuple._i;
    )
    )

desugar-expr-block
:: Vec<Statements> -> (
    C declare __blockn,
    Scope (
        Statements[0..n-1],
        C assign __blockn = Statements[n]
    )
    )

desugar-condmatch
:: Vec<(Cond, Expr)> ->
    if default in cond then (
        for cond, expr in condmatch where cond != deafult (
            C if (cond) { expr } else
        ),
        expr in condmatch where cond == default
    ) else (
        for cond, expr in condmatch[0..n-1] (
            C if (cond) { expr } else
        ),
        expr of condmatch[n]
    )


desugar-implicit-return
:: Vec<Statements> -> (
    C declare __return,
    Statements[0..n-1],
    C assign __return = Statements[n],
    C return __return
    )