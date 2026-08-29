# zealang

once you Zea it, you won't want to C it.

Zea is a C-like language whose main goal is to reduce the boilerplate required to simulate
features of more modern languages

The syntax looks quite similar to Rust, mostly because I like Rust's syntax :D

# working features

- emitting global integers into the object files
- returning integer -literals, -locals and -globals from `main()`
- thats about it right now

# how to use
- clone the project
- run `cargo build --release`
- run `./target/release/zea-driver --help` to show CLI usage

Some features include:
## tuple-types and first-class syntax for them
### tuple-types
`fn make-coord(x: F32, y: F32) -> (F32,F32) { (x,y) }`

### tuple-destructuring:
to destructure a tuple, a tuple-pattern is used:
`@(x,y) : (F32,F32) = coord;`
`@(a,b,(c,d)) := some-tuple;`

The pattern has the following grammar rule:
```ebnf
tuple-pattern ::= '@' tuple-pattern-inner
tuple-pattern-inner ::= '(' pattern (',' pattern)* ')'
pattern ::= tuple-pattern-inner | identifier
```


## Some ergonomics borrowed from modern languages

### order of declaration is not significant
no more forward declarations!

```Rust
  fn foo() -> U32 {bar()}
  
  fn bar() -> U32 {3}
// this is allowed just fine!
```

### blocks-as-expressions and tail-returns
```Rust
fn square(x: U32) -> U32 {
    x * x // tailing expression in a block is treated as return
}

fn read-to-string(path: String) -> *String {
    // option type and early return
    f : File = fs:open(path) else return nil; 
    buffer := String:new();
    fs:drain-into!(buffer, f);
    buffer
}

fn foo() -> U32 {
    a := {
        b:= 3; 
        b
    };
    a + 1;    
}
```

## defining datatypes
Zea allows three kinds of datatypes: structs, tags and unions,
with plans to add ergonomics for tagged-unions (i.e. Rust enums) later.

structs are compound-datatypes which can contains any amount of field of mixed types.

tags are like C-enums, literally just integers with a name, except that they are in a namespace now!

Unions allow developers to read and write to a single variable as different types.
They are identical to C-unions, except that type-punning (explained later) is well-defined now.

#### Structs
to define a struct, use the `struct` keyword, define fields by declaring variables without the `= value` part:
```
struct Foo {
    field1: U32,
    field2: Bool,
    field3: String*
}
```

Instantiating a struct looks similar to defining one, except that you *do* include the `= value` part,
it is not necessary to specify the type.

```
string := "meow";

foo := Foo {
    field1 := 3,
    field2 := true,
    field3 := string&
};
```

#### Tags
To define a tag, use the `tag` keyword, by default, the tags are assigned integers in increasing order, starting from 0.
You may assign unique values to tag variants. To determine  what the integer value of a tag variant is, you can follow the simple rule:
the value of a tag variant is the one it is explicitly given. If it is not explicitly given one, it is one more than the one above it:

```
tag HTTPCode {
    Ok: 202, // 202, duhh
    Okp1, // 203, because it has no assigned value
    Forbidden: 403 // 403, duhh
    Error, // 404
} 
```

to get the value of a tag as an integer, you can use the `discriminant()` method that each tag datatype gets automatically:
```
code := HTTPCode:Ok;
code_as_int := code:discriminant();
if code_as_int == 202 {
    stdout:print-line("its okay bro");
}
else {
    stdout:print-line("wtf");
}
```

#### Unions
To define a unions, use the `union` keyword, field are declared the same way a struct does,
instantiating a union is done by treating the fields as contructor-functions:
```
union U64orF64 {
    uint: U64,
    float: F64
}

u := U64orF64:uint(3);
f := U64ofF64:float(3.1415);
```

To write to a union, you must select a variant to write to, and provide a value of that same type.

Unions are meant to be very primitive,
as such, there is no way to check which field was last written to or read from.
The size of a union is that of its largest variant, the above example would have a size of 64 bits.

Unions can be used to read bits of one type as if they were of another, this is called *type-punning*

See the below example, where the famous [Fast Inverse Square Root from Quake](en.wikipedia.org/wiki/Fast_inverse_square_root)
is implemented using type-punning.

```
union Helper {
    f: F64,
    u: U64,
}
fn Q-rsqrt(f: F64) -> F64 {
    x2 := f * 0.5;
    // construct as F64
    i := Helper:f(f);
    
    // then read as U64 and write to the U64 variant
    i.u = (0x5f3759df - (i.u >> 1));

    // then read and copy the value of the F64 variant, which is now modified 
    y := i.fptr; 

    y = y * (3.5 - (x2 * y * y)); // euler shit
    y // tadaaaaaaa we have (1 / sqrt(f))
}
```

#### Tagged unions
As of right now, tagged unions are a construct developers must build themselves, leading to lots of boilerplate.
We plan to implement tagged unions in a fashion similar to Rust somewhere in the future. Probably using some sort of syntax sugar.

### Namespaces
A file may declare a module, which contains imports, exports, datatypes, functions and globally-scoped identifiers.

```
// in file FooMod.zea
module FooMod

exports {
    fn foo;
    bar;
    struct Baz;
}

fn foo(a: U32) -> U32 {a * 2}
bar: U32 = 3;

struct Baz {
    baz_field: U32,
}
```
Which can then be imported in another module
```
// in file Chicken.zea
module Chicken

imports {
    FooMod:foo;
    FooMod:bar;
    FooMod:struct Baz;
}

struct Chicken {
    eggs_laid: U32,
}

fn Chicken:new() -> Chicken {
    Chicken {
        .eggs_laid: 0
    }
}

fn Chicken:lay-egg(self: Chicken) {
    self.eggs_laid += 1;
    stdout:print-line("pock pock");
}

fn main(argv: [String]) -> U8 {
    let c := Chicken:new();
    c:lay-egg();
} 
```

The `:` operator has two uses, as a namespace-accessor, or alternatively as an instance-accessor.
It is always known from context which one is meant, as Modules and Types always start with a capital letter,
while identifiers always start with a lowercase letter.
`Foo:bar()` (module-call) calls the `bar()` function defined in the `Foo` module.

`foo:bar()` (instance-call), in the case that `foo` is of type `Foo`,
this calls the same function, passing the variable `foo` as the first argument.

In the case of an instance-call, Zea will infer which function to call by infering the type of the instance variable.
You may then notice there are now two ways to operate on instances of types:
```
module Foo
exports {struct Foo;}

struct Foo {}

fn bar(foo: Foo) {...}

foo := Foo {}; // instantiate some instance of the Foo type

// we can do an instance-call:
foo:bar();

// or, alternatively a module-call:
Foo:bar(foo);
```
the use of instance-calls is encouraged.

## The Type System
Zea approaches specifying types in a manner more similar to functional Rust, where types have no modifiers,
instead a 'modified' type is considered a separate type.

Take for instance the `unsigned` modifier in C, compared to the `I32` vs. `U32` types in Rust.
This is done to make both the grammar of the language less ambiguous, and to make the code easier to parse for humans.

Zea uses the `var: Type = value` syntax instead of the `Type var = value` syntax. This makes omitting types more pretty: `var := value`,
and makes the grammar rule for declarations simpler.

As you might have inferred (hah, get it) from the above, Zea features type inference by a simplified Hindley Milner type system, Zea features inference without generics (yet?),
which makes the type-checker simple (thank god).
