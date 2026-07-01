# zealang

once you Zea it, you wont want to C it.

Zea is a preprocessor for C whose main goal is to reduce the boilerplate required to simulate
features of more modern lagnuages

Some features include:
## tuple-types and first-class syntax for them
### tuple-types
`fn make-coord(x: F32, y: F32) -> (F32,F32) { (x,y) }`

### tuple-destructuring:
to destructure a tuple, a tuple-pattern is used:
`@(x,y) :(F32,F32) = coord;`
`@(a,b,(c,d)) := some-tuple;`

The pattern has the following grammar rule:
```ebnf
tuple-pattern ::= '@' tuple-pattern-inner
tuple-pattern-inner ::= '(' pattern (',' pattern)* ')'
pattern ::= tuple-pattern-inner | identifier
```


## Some ergonomics borrowed from modern languages
### tagged-unions

WIP

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

The `:` operator has two uses, as a namespace-accessor, and as an instance-accessor.
The language knows which one to use as Modules and Types always start with a capital letter,
and identifiers always start with a lowercase letter.
`Foo:bar()` (module-call) calls the `bar()` function defined in the `Foo` module.

`foo:bar()` (instance-call), in the case that `foo` is of type `Foo`,
calls this same function, passing the variable `foo` as the first argument.

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

### defining datatypes
Zea allows developers to define three kinds of datatypes: structs, tags and unions,
with plans to add ergonomics for tagged-unions later.

structs are compound-datatypes which can contains an amount of field of mixed types.

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
