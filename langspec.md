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
### Struct
### Union
### Option
### Result
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

<numeric literal> =
    regex/(0d)?[_0123456789]+/i
|   regex/0x[_0123456789abcdef]+/i
|   regex/0b[01][_01]*/i

```

