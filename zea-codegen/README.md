# zea-codegen

This module contains the logic for building QBE IL out of the THR IL.

It lowers globals into typed byte-strings as specified by the QBE spec,
and demangles symbols using the following convention:
# symbol mangling
## function-locals
locals are mapped to QBE temporaries (`%tempname`)
a local variable `a` within a function `f` within module `m` is mangled as follows:
`L_[m]_[f]_local_[a]` (the percent for QBE temporaries).

Any intermediate expression temporaries get some unspecified integer as their symbol, i.e.
```Rust
module main
fn f() {
a: u8 = 3 + 4;
}
```
could be mangled into the following QBE assignments:
```qbe
%0 = imm 3
%1 = imm 4
%L_main_f_local_a = add %0, %1
```
the current implementation actually mangles intermediate expressions the same way as locals,
but uses a counter instead of a variable name:
```qbe
%L_main_f_local_0 = imm 3
%L_main_f_local_1 = imm 4
%L_main_f_local_a = add %L_main_main_f_local_0, %L_main_main_local_1
```
This mangling scheme cannot cause naming conflicts as expression identifiers cannot consist of
only numbers.

## global variables
globals are mapped to QBE globals.
a global variable `g` within module `m` is mangled as follows:
`G_[m]_global_g`

There may not be multiple globally scoped symbols with the same name,
so using only a `global_` prefix should be enough, as we need not differentiate between
functions and globals as we assume their names are unique.


## datatypes
QBE allows for naming aggregate types. These are like structs
but do not record their field-offsets, nor do they insert padding implicitly
(a-la `#[repr(packed)]`. this is no problem
as the THR will have desugared member-accesses into pointer-arithmetic,
as will it have inserted padding if needed.
a datatype `d` in a module `m` will be mangled as follows:
`D_[m]_d`

## functions
Functions are mapped to QBE functions (wow! who would have thought!).
Zea does not allow function overloading. It currently also does not feature
generics, but it might in the future.

### non-generic functions
A non-generic function `f` within module `m` is mangled as follows:
`F_[m]_c_[f]` where the c stants for 'concrete'.

### generic functions
Zea does not yet feature generics, so this is not really relevant right now.

A generic function `f` in module `m` with monomorphised type-parameters `T,U,V`
(meaning that `T` might be `U8` or some other concrete type)
needs to have its type-parameters mangled before `f` can be mangled.
There are some rules; any type-parameter that is a primitive type (`U8`, `Bool`, `*U8` etc.)
Can be 'mangled' just by using the type name. the unit type `()` is mangled to `Unit`.

Any aggregate type `T` is first mangled into `M[T]`, see the datatypes section.

The function name is then mangled as follows:
`F_[m]_gN<[M[T] for each parameter T].join(',')>_[f]`
that is, we insert a comma-delimited list of each mangled parameter in-between angle brackets
before the funtion name. The list is prefixed by the amount of parameters `N`.

An example:
```Rust
module main
struct S { ... }
fn foo<T,U>(t: T, u: U) { ... }

// with some usage like
foo<U8, S>(3, S:new())
// and
foo<S, ()>(S:new(),())
```
would have the `f<U8, S>` occurence be mangled into:
`F_main_g2<U8,D_main_S>_foo`

and

`F_main_g2<D_main_S,Unit>_foo` respectively

Note that these mangling formats might change at any time,
as I have probablt forgot about some edge case or something
