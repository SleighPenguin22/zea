# The Zea language specification

Version 0.1
(28 Feb 2026)

# Philosophy

# Program Structure

## Program Entry Point

# Variables and Expressions

# Types

## Scalar Types

### Integers

### Floating-point

### Boolean

## Compound types

# Compiler options

## flags
- `--print-mir`, `--print-mir=file`: print the AST after all static analysis is performed, to stdout, stderr or a specified file.
- `--loglevel=LEVEL`: specify the logging level, where `LEVEL` is one of `[ERROR, WARN, INFO, DEBUG, TRACE]` (from least- to most-verbose)

## EXTENSION: Compiler subcommands
The compiler can also setup a project skeleton for you 

## EXTENSION: Compiler directives
A compiler directive is an annotation above some piece of code that has the form `@ [directive-name] [directive option]`

Some directives are applicable to the compilation of a whole module, while others apply to specific datatypes or functions within a module.

All directives have a default value,
These are the values that are used when compiling using the default flags and when omitting directives in-code.

To change the defaults within a module, you may supply them above the module declaration like so:
```
@ layout-reorder = false
module main
...
```

This will apply the directive to all applicable items within that module.

To change the defaults or directives when compiling multiple modules, directives can be supplied after the `--@` subcommand.
```
zea compile a.zea b.zea --@ layout-reorder=false target-arch=x86_64
```
This will compile both `a.zea` and `b.zea` as if their declaration had `@ layout-reordering = false` above them.

alternatively, a path to a `.toml` file containing directives can be supplied instead:
```
// flags.toml
layout-reordering = false
target-arch = "x86_64"
...
// flags.toml

> zea compile a.zea b.zea --@ file=flags.toml
```

The compiler will search for a `flags.toml` file within the same directory that the compiler is called from,
if it finds such a file, it will apply those flags to the compilation.

These directives instruct the compiler to do something at compile-time, such as:
- `@ layout-reorder = true|false` for specifying struct layout ordering
- `@ target-arch = [architecture]` for specifying the target architecture
- `@ alignment = N` for specifying the alignment of a struct in bytes
- `@ inline` for inlining
- `@ hot` and/or `@ cold` for hinting hot- or cold branches
- `@ unreachable` for specifying unreachable code branching code that must otherwise be exhaustive
- `@ eliminate-dead-code = false` for specifying if code may or may not be optimized out, even if it never called (default = true)

Supplying an non-existent directive, or supplying a directives with an invalid value such as `@ alignment = shimmadingle` will result in a compilation failure.

## EXTENSION: profiles
A `profiles.toml` file containing profiles can be supplied, along with a profile upon compilation.
```
// profiles.toml

default-profile = debug

[optimize]
Olevel = 3
strip-debug = true
debug-assert = "ignore"
crash-unreachable = false
zero-initialize = false

[debug]
Olevel = 0
strip-debug = false
debug-assert = "keep"
```

a profile can be chosen at compile-time using the `--profile=[profile]` flag, if it is omitted, the `default-profile` form the profiles is used.

## Typing Rules

| binop                              | typeof(lhs) | typeof(rhs) | inferred type                             |
|------------------------------------|-------------|-------------|-------------------------------------------|
| `+`,`-`,`*`,`/`,`%`, `&`, `\|`,`^` | `Int(s1)`   | `Int(s2)`   | `Int(max(s1,s2))` (zero extension rule)   |
| `+`,`-`,`*`,`/`,`%`                | `Float(s1)` | `Float(s2)` | `Float(max(s1,s2))` (zero extension rule) |
| `&&`,`\|\|`,`^^`                   | `Bool`      | `Bool`      | `Bool`                                    |
| `[]` (indexing)                    | `[t]`       | `Int(s)`    | `t`                                       |
| `.` (member access)                | `t`         | ...         | `t::m` (struct lookup)                    |

| unop | typeof(arg) | inferred type |
|------|-------------|---------------|
| `-`  | `Int(s)`    | `Int(s)`      |
| `-`  | `Float(s)`  | `Float(s)`    |
| `~`  | `Int(s)`    | `Int(s)`      |
| `~`  | `Bool`      | `Bool`        |

## type conversion rules/conditions
- An integer type can always be cast into a wider integer type of the same sign: `u8 -> u16|u32|u64` and `i8 -> i16|i32|i64`.
- An unsigned integer type can always be case to a wider signed type: `u8 -> i16|i32|i64`
- A `signed -> unsigned` cast must always be explicit, and is identical to a bit-reinterpretation (i.e. `-1 -> 255`)
- A boolean may always be cast into any integer type, where `true -> 1` and `false -> 0`
- An `integer -> boolean` cast must be explicit, and is equal to checking for non-equality with `0`, i.e. `int != 0`
- An integer cast to a more narrow integer type must be explicit, and is identical to truncation.
- An integer cast of the form `Uw1 -> Iw2` or `Iw1 -> Uw2` where `w1 < w2` (a widening cast that also changes the sign)
  will first perform the sign-cast, then the widening cast: `I8 -> U16 === I8 -> U8 -> U16`
- An `F32 -> F64` cast is always allowed, the other way around must be explicit however.
- A `boolean -> float` cast must be explicit, where `true -> 1.0f64` and `false -> 0.0f64`
- Pointer to different types may never be cast into one another: `*U8 -> *I8` will always throw an error,
  use a union if you wish to achieve behaviour like this

The standard library provides a module called bit-reinterpret,
which will be populated at compile-time with function that bit-cast between types. Cast functions will only be synthesized for types of equal width.

## Struct size and field ordering
Struct will have an alighment equal to their widest field.
The size `S` of a struct is calculated as follows:
- call the alignment of the Struct `A`
- call the sum of the size of the fields `Sumf`
- find the smallest `n`in `n * A` such that `S >= Sumf`
- the size `S` is then `n * A`
i.e. the smallest multiple `n` such that `n * A >= Sumf`
Struct fields are ordered largest to smallest by default, as to minimize the padding required.

#### EXTENSION: opt-out of field-reordering
The developer may specify if layout-reordering should be applied by directing the compiler using `@ layout-reorder = [boolean]`,
whereas the default is `@ layout-reorder = true`. The directive subcommand option `layout-reorder=(true|false)` may be set to change the default. 


## stupid C shit you could do
What happens if you where to do the stupid shit you could do in C?
- unreachable code
    - reaching code marked `@ unreachable` when `@ crash-unreachable = true` (the default) will immediatly crash the program with an error,
    - reaching code marked `@ unreachable` when `@ crash-unreachable = false` will depend entirely on the generated binary and is considered undefined,
      it may lead to memory corruptions or explosions or something idk, be careful.
- bit-casting between types of different sizes using `bit-reinterpret<val, Type>` depends:
    - casting to a wider type will zero-extend the value with most-significant bytes
    - cating to a narrower type will truncate the excess bits
- dereferencing a NULL pointer will immediatly crash the program with an error
- initialization
    - allocated memory is always initialized to 0 when the `@ zero-initialize = true` (the default) is set
    - allocated memory is left unitialized when `@ zero-initialize = false`



# Grammar

```bnf
<module> = 
    "module" <expr ident>
    ( "imports" "{" <expr ident>,+  "}" )?
    ( "exports" "{" <expr ident>,+  "}" )?
    <module items>
    
<module items> =
    <function definition> <module items>
|   <initialisation> <module items>

<initialisation> =
     <assignment pattern> ":" <type specifier>? "=" <expression> ";"

<reassignment> =
     <expr ident> "=" <expression> ";"


<assignment pattern> =
    <expr ident> // simple assignee
|   "(" <expr ident>,+ ")" // unpacking assignee

<type specifer> =
    <type ident> // basic type
|   <type ident> "*" // pointer type
|   "[" <type ident> "]" // array type

<function definition> =
    "func" <expr ident> 
    "(" <typed ident>,* ")"
    "->" <type specifier> 
    <statement block>

<typed ident> =
    <expr ident> ":" <type specifier>

<struct definition> = 
    "struct" <type ident> "{"
        <typed identifier>,+
    "}"

<if branch> =
    "if" <expression> <statement block>
|   "if" <expression> <statement block> "else" <statement block>


<expression> = 
#precedence 0
    <expression> "||" <expression>
#precedence 1
    <expression> "^^" <expression>
#precedence 2
    <expression> "&&" <expression>
#precedence 3
    <expression> "|" <expression>
#precedence 4
    <expression> "^" <expression>
#precedence 5
    <expression> "&" <expression>
#precedence 6
    <expression> "==" <expression>
|   <expression> "!=" <expression>
#precedence 7
    <expression> "<" <expression>
|   <expression> ">" <expression>
|   <expression> "<=" <expression>
|   <expression> ">=" <expression>
#precedence 8 (left associative)
    <expression> "<<" <expression>
|   <expression> ">>" <expression>
#precedence 9 (left associative)
    <expression> "+" <expression>
|   <expression> "-" <expression>
#precedence 10 (left associative)
    <expression> "*" <expression>
|   <expression> "/" <expression>
|   <expression> "%" <expression>
#precedence 11
    "!" <expression>
|   "-" <expression>
|   "~" <expression>
#precedence 12
    <expression> "." <expr ident>
|   <expression> "[" <expression> "]"
|   <function call>
#precedence 13
    <expr ident>
|   <integer literal>
|   <float literal>
|   "true" | "false"

<statement> =
    <initialisation>
|   <function call> ";"
|   "return" <expression> ";"
|   <reassignment>
|   <statement block>

<function call> =
    <expr ident> "(" <expression>,* ")"

<statement block> =
    "{" <statement>+  <expression>? "}"

<numeric literal> =
    regex/(0d)?[_0123456789]+/i
|   regex/0x[_0123456789abcdef]+/i
|   regex/0b[01][_01]*/i

<float literal> =
#helper <sign> = "+" | "-"
#helper  <e> = "e" | "E"
#helper <exponent> = <e> <sign>? regex/[0123456789]+/
    <sign>? regex/[0123456789]+/ "." <exponent>?
|   <sign>? "." regex/[0123456789]+/ <exponent>?
|   <sign>? regex/[0123456789]+/ "." regex/[0123456789]+/ <exponent>?
    
```
