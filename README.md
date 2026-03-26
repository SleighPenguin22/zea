# zealang

once you Zea it, you wont want to C it.

Zea is a preprocessor for C whose main goal is to reduce the boilerplate required to simulate
features of more modern lagnuages

Some features include:
## tuple-types and first-class syntax for them
### tuple-types
`fn make-coord(x: F32, y: F32) -> (F32,F32) { (x,y) }`

### tuple-destructuring:
`(x,y) :(F32,F32) = coord;`
`(a,b,(c,d)) := some-tuple;`

## Some ergonomics from modern languages
### tagged-unions

WIP

### order of declaration is not significant
no more forward declarations!

```Rust
  fn foo() -> U32 {bar()}
  
  fn bar() -> U32 {foo()}
```

### blocks-as-expressions and tail-returns
```Rust
fn square(x: U32) -> U32 {
    x * x // tailing expression in a block is treated as return
}

fn read-to-string(path: String) -> *String {
    // option type and early return
    f : File = fs:open(path) ?> nil; 
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

### Lisp-style identifiers
```Rust
fn is-even?(x: U32) -> Bool {
    x % 2 == 0
}

// Bangs are encouraged for mutating functions,
// Annotating the parameters that get mutated
fn extend!(a!: [I32], b: [I32]) -> [I32] {
    [I32]:extend-capacity-by!(b.len, a!)!;
    [I32]:copy-into!(a!, b);
    a!
}

fn extend(a: [I32], b: [I32]) -> [I32] {
    result := [I32]:new-reserve(a.len + b.len);
    [I32]:copy-into!(result[0], a);
    [I32]:copy-into!(result[a.len], b);
    result
}
```