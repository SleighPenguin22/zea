# zea-irs

This crate contains the various Intermediate representations used in the Zea compiler

As of now, there are two representation used in the frontend:
- Immediate Parsed Representation (IPR), a tree-like representation
	this IR represents the code that the parser outputs. It performs three passes,
	namely:
	- desugaring of packed assignments
	- resolving the scope of symbol-usages
	- type-checking and inserting missing type-annotations
- Typed Highlevel Representation (THR), a flat, interned representation
	this representation is a struct-of-arrays, it inherits all the scope- and type- info
	from the IPR, and calculates struct -aligments, -sizes, -offsets.
	It still contains assignment and expressions as you would see in the IPR.

Then the THR is compiled into QBE IR. As of now, the `qbe` crate is used, which allows us
to call `std::fmt::Display` on the `qbe::Module` to get a correct QBE IL format, which we can
write to a file and pass to the `qbe` binary, which converts it to assembly.

My plan for the future is to rewrite QBE in Rust, as I originally wanted to fork QBE and add
a native way to generate the IL using C bindings. It turns out that the QBE code does not allow
for that without a major refactor (its parser is tightly intertwined with the IL itself).
Rewriting QBE in Rust would allow me to use it as a library
with nice builder patterns and the like, and to also implement
a driver with a parser a-la the current QBE.

When this rewrite is usable, the THR can use these builders to directly synthesize
the QBE IL, which can then emit an ELF object file without needing to run a separate executable.
I suspect this will improve the compiler-debugging experience, and it cleans up the `zea-driver`.

