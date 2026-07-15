# Zea MIR Node Structure Proposal

> A verbose reference document. Naming convention: I will use "QBE" to refer to
> the QBE IL backend (<https://c9x.me/compile/>) and "LLVM" for the LLVM IR
> backend. When a design choice differs between the two, both are noted.

---

## Part 1 — Type System & Core Identifiers

### 1.1 Why MIR needs its own type system

The HIR has `HIRTypeSpecifier` (`zea-ast/src/zea/mod.rs:594-617`) — an abstract,
structural type system. It carries no layout information: no byte sizes, no
alignments, no field offsets. The frontend does not care about layout. The MIR,
however, must know the exact memory footprint of every value because it will be
lowered to QBE (or LLVM), both of which require explicit, sized types on every
instruction.

The HIR enum is:

```rust
pub enum HIRTypeSpecifier {
    NonScalar(String),       // named struct type
    Unit,                    // ()
    Bool,                    // boolean
    Integer { width: usize, signed: bool },
    Float { width: usize },
    Pointer(Box<HIRTypeSpecifier>),
    ArrayOf(Box<HIRTypeSpecifier>),
    Never,                   // diverging
}
```

This is insufficient for MIR for three reasons:

1. **No sizes.** `Integer { width: 8 }` conceptually has width 8 bits = 1 byte,
   but what about alignment? What about `ArrayOf` — what is the total byte size?
2. **No field offsets.** `NonScalar("Foo")` tells you the struct name but not
   where each field lives in memory. Codegen needs these offsets to emit GEP or
   index calculations.
3. **QBE types are few and simple.** QBE has four base types: `w` (word,
   32-bit), `l` (long, 64-bit), `s` (single, 32-bit float), `d` (double,
   64-bit float). The HIR has 8-bit through 64-bit integers, but QBE only does
   32-bit and 64-bit operations natively. So `u8` must be promoted to `w` for
   arithmetic and truncated on stores. The MIR type system should reflect this
   reality.

### 1.2 Core ID types

All MIR nodes are identified by flat integer handles. This is the foundation of
the DAG: multiple instructions can reference the same `MIRValueId`, forming a
directed acyclic graph of producers and consumers. No `Box<...>` nesting, no
recursive ownership.

```rust
/// Unique identifier for a MIR value.
///
/// Every MIR instruction produces exactly one value, identified by a MIRValueId.
/// Block parameters (phi destinations) also get MIRValueIds.
/// Function arguments also get MIRValueIds.
///
/// A MIRValueId can be referenced by zero or more consumers. If zero, the value
/// is dead and can be eliminated. If multiple, we have a DAG.
///
/// This is equivalent to:
///   - LLVM's `Value*` pointer identity
///   - Cranelift's `Value` newtype index into the DataFlowGraph
///   - QBE's `%temporary` numeric suffix
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct MIRValueId(pub u32);

/// Unique identifier for a basic block.
///
/// Blocks are nodes in the control-flow graph (CFG). A block contains a
/// sequence of instructions and exactly one terminator that transfers control
/// to another block (or returns).
///
/// Equivalent to:
///   - LLVM's `BasicBlock*`
///   - QBE's `@label` syntax
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct MIRBlockId(pub u32);

/// Unique identifier for a concrete type in the MIR type table.
///
/// The module owns an IndexMap<MIRTypeId, MIRType>. Every MIRInstruction carries
/// a MIRTypeId for its result type, and every alloca/load/store references
/// types by their MIRTypeId.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct MIRTypeId(pub u32);
```

### 1.3 Concrete MIR types

```rust
/// A fully-laid-out type with size, alignment, and (for structs) field offsets.
/// This is what the codegen backend consumes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MIRType {
    pub id: MIRTypeId,
    pub kind: MIRTypeKind,
    /// Size in bytes. Computed during HIR→MIR lowering via layout rules.
    pub size: usize,
    /// Alignment in bytes. For scalars this equals size (capped at 8 for
    /// word, 8 for long, 4 for single, 8 for double). For structs this is
    /// the maximum alignment among all fields, per Zea's layout spec.
    pub alignment: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MIRTypeKind {
    /// The unit / void type. Zero-sized.
    ///
    /// QBE has no void type — a function returning () just omits the `ret`
    /// value. In LLVM, this maps to `void`.
    ///
    /// Unit values carry no data. An instruction producing Unit produces a
    /// /marker/ value — it is essentially a statement that happens to sit in
    /// the instruction stream. Store/load of Unit is a no-op.
    Unit,

    /// Word-sized integer (32-bit on QBE, pointer-sized on LLVM).
    ///
    /// This is the /universal integer carrier/ in Zea's MIR. It is used for:
    ///
    ///   - **Booleans** (0 = false, 1 = true). QBE comparison ops (ceqw,
    ///     csltw, etc.) all produce a `w` result. Boolean operations (&&, ||)
    ///     are lowered to branches + phi nodes, not arithmetic.
    ///
    ///   - **Small integers** (u8, i8, u16, i16, u32, i32). These are all
    ///     promoted to `Word` for arithmetic. The promotion is implicit:
    ///     a `load` of a u8 loads a byte and zero-extends (or sign-extends)
    ///     to `Word`. A `store` of a `Word` to a u8 slot truncates.
    ///
    ///   - **Comparison results.** Every cmp instruction (CmpEq, CmpSLt,
    ///     CmpFLt, ...) produces a `Word` that is either 0 or 1.
    ///
    ///   - **Pointer arithmetic temporaries.** In QBE, pointer arithmetic
    ///     (`add %ptr, %offset`) requires the offset to be a `l`/`w`.
    ///
    ///   Maps to QBE type `w`.
    ///
    ///   Maps to LLVM type `i32` (or `i64` on 64-bit targets — but since QBE
    ///   is our primary target, we use 32-bit words).
    Word,

    /// 64-bit integer.
    ///
    /// Used for u64 and i64 HIR types. All u64/i64 arithmetic stays in Long.
    /// When mixing Word and Long (e.g. u32 + u64), the Word is zero-extended
    /// (ZExt) or sign-extended (SExt) to Long.
    ///
    /// Maps to QBE type `l`.
    /// Maps to LLVM type `i64`.
    Long,

    /// 32-bit float (IEEE 754 binary32).
    ///
    /// Maps to QBE type `s`.
    /// Maps to LLVM type `float`.
    Single,

    /// 64-bit float (IEEE 754 binary64).
    ///
    /// Maps to QBE type `d`.
    /// Maps to LLVM type `double`.
    Double,

    /// A pointer to some other type.
    ///
    /// In QBE, pointers are not strongly typed — a pointer to `w` and a
    /// pointer to `d` are both just `l` at the machine level. However, we
    /// track the pointee type so we can:
    ///   1. Emit the correct load/store width (loadw vs loadd, etc.)
    ///   2. Compute field offsets for struct member access
    ///
    /// On 64-bit systems, pointers are 8 bytes. On 32-bit they would be 4.
    Pointer(MIRTypeId),

    /// A fixed-size array of `count` elements of type `elem`.
    ///
    /// Total size = count * elem.size. Alignment = elem.alignment.
    ///
    /// Array indexing `arr[i]` in MIR is:
    ///   %base = ...                       ; pointer to array start
    ///   %idx  = ...                       ; index (Word)
    ///   %offset = mul %idx, elem_size     ; byte offset
    ///   %ptr   = add %base, %offset       ; pointer to element i
    ///   %val   = load %ptr                ; value at arr[i]
    Array { elem: MIRTypeId, count: usize },

    /// A product type (named struct).
    ///
    /// `fields` is a vector of (field_name, field_type_id, byte_offset).
    /// The byte_offset is computed during HIR→MIR lowering using Zea's
    /// struct layout rules:
    ///   - Alignment = max alignment among all fields
    ///   - Fields are reordered largest-to-smallest by default to minimize
    ///     padding (controlled by `@layout-reorder` directive)
    ///   - Total size is the smallest multiple of alignment that fits all
    ///     fields (i.e. padded to alignment boundary)
    ///
    /// Struct access in MIR uses integer field indices (computed from the
    /// HIR's string-based MemberAccess during lowering):
    ///
    ///   %base   = ...                     ; pointer to struct start
    ///   %fptr   = fieldptr %base, 2       ; pointer to field index 2
    ///   %val    = load %fptr              ; value at that field
    ///
    /// QBE has no struct type. Structs are lowered into sequences of
    /// per-field allocas and loads/stores, or into a flat byte array with
    /// index arithmetic.
    Struct {
        name: String,
        /// (field_name, field_type_id, byte_offset_in_struct)
        fields: Vec<(String, MIRTypeId, usize)>,
    },
}

impl MIRType {
    /// Convert this MIR type to the QBE base type character and return the
    /// corresponding byte size. Used during codegen to pick the right
    /// instruction suffix (loadw, loadl, loads, loadd).
    ///
    /// Panics on aggregate types — they don't map to a single QBE base type.
    pub fn qbe_base(&self) -> (&'static str, usize) {
        match self.kind {
            MIRTypeKind::Unit => unreachable!("unit has no QBE representation"),
            MIRTypeKind::Word  => ("w", 4),
            MIRTypeKind::Long  => ("l", 8),
            MIRTypeKind::Single => ("s", 4),
            MIRTypeKind::Double => ("d", 8),
            MIRTypeKind::Pointer(_) => ("l", 8),
            MIRTypeKind::Array { .. } => panic!("array has no QBE base type"),
            MIRTypeKind::Struct { .. } => panic!("struct has no QBE base type"),
        }
    }
}
```

### 1.4 HIR → MIR type mapping

| HIR TypeSpecifier           | MIR TypeKind                    | QBE  | Reason                                                        |
|-----------------------------|---------------------------------|------|---------------------------------------------------------------|
| `Bool`                      | `Word`                          | `w`  | QBE has no bool; comparisons produce `w`. Reduces coercion.   |
| `Integer { width: 8..32 }`  | `Word`                          | `w`  | Promoted to word for arithmetic; truncated on stores.         |
| `Integer { width: 64 }`     | `Long`                          | `l`  | Native 64-bit integer support in QBE.                         |
| `Integer { width: 64, signed }` | `Long`                      | `l`  | Signedness is captured by instruction choice (sdiv vs udiv).  |
| `Float { width: 32 }`       | `Single`                        | `s`  | IEEE 754 binary32.                                            |
| `Float { width: 64 }`       | `Double`                        | `d`  | IEEE 754 binary64.                                            |
| `Unit` / `()`               | `Unit`                          | (none) | Zero-sized marker. No QBE type needed.                       |
| `Pointer(T)`                | `Pointer(T_mir)`                | `l`  | Pointers are 64-bit longs in QBE.                             |
| `ArrayOf(T)`                | `Array { elem: T_mir, count }`   | (composite) | Decomposed into index arithmetic.                         |
| `NonScalar(name)`           | `Struct { name, fields }`        | (composite) | Lowered field-by-field.                                       |
| `Never`                     | `Unit`                          | (none) | Diverging code; the unit value is never actually used.        |

### 1.5 Why booleans are word-sized (in detail)

The language spec says booleans are a scalar type. Conceptually they occupy
1 bit. But in the MIR they are word-sized (32-bit) integers where only 0 and 1
are valid. The reasons:

1. **QBE has no boolean type.** QBE's comparison instructions (`ceqw`, `csltw`,
   `cnew`, etc.) produce a `w` (word) result that is either `0` or `1`. There
   is no QBE type for a 1-bit value. If we introduced a hypothetical `Bool`
   type in MIR, every branch (`jnz` in QBE) would still need to test a
   word-sized value, forcing us to emit a `widen` or `cast` instruction before
   every branch. Making bools word-sized eliminates this: the comparison result
   is already the right width for branching.

2. **LLVM uses `i1` but aligns to 1 byte.** LLVM's `i1` is 1-bit in SSA form
   but when stored to memory via `alloca i1`, it occupies at least 1 byte.
   LLVM inserts implicit `zext`/`trunc` around memory operations. This is
   essentially the same model — the type system pretends it's 1-bit but the
   backend treats it as 1-byte. In our MIR, we skip the pretense and make it
   explicit: bools are words.

3. **Word-sized bools eliminate truncation chains.** Consider `a && b` where
   `a: bool` and `b: bool`. If bools were `u8`:
   ```
   %a = load u8        ; load bool as u8
   %aw = zext %a -> w  ; widen to word for comparison
   %t  = cmpne %aw, 0
   jnz %t -> rhs, false_arm
   rhs:
   %b  = load u8
   %bw = zext %b -> w
   %r  = cmpne %bw, 0
   ; ... now %r is word-sized, but b is u8-sized
   ; store to b's slot needs a trunc
   ```
   With word-sized bools, all these `zext`/`trunc` pairs disappear.

4. **The spec's `bool + int → int` coercion rule still works.** In the type
   checker, a boolean coerced to an integer yields the integer. In our MIR,
   the bool is already word-sized, so the coercion is a no-op — reuse the same
   `MIRValueId`. The `0`/`1` encodings align perfectly.

---

## Part 2 — MIR Instructions (the DAG Nodes)

### 2.1 Design philosophy — flat, enumerated, single-result

The HIR represents expressions as nested trees:

```
HIRExpression
├── BinOpExpr ➜ Box<HIRExpression>, Box<HIRExpression>
│   ├── IntegerLiteral(1)
│   └── BinOpExpr
│       ├── IntegerLiteral(2)
│       └── Ident("x")
```

This is a tree with `Box<...>` for ownership. It is not a DAG because each
subexpression is owned by exactly one parent. Two parents cannot share the
same `1` literal.

The MIR breaks each node into a **flat instruction indexed by MIRValueId**.
Each instruction produces exactly one value. Each instruction may reference
zero or more other values by their MIRValueId. This forms a DAG:

```
%1 = ConstInt(1)        //    %1
%2 = ConstInt(2)        //   /   \
%3 = Load(x)            //  %2   %3      <- %1 shared by both consumers
%4 = Add(%2, %3)        //   \   /
%5 = Add(%1, %4)        //    %4
                        //     |
                        //    %5
```

The key property: **one definition, many uses.** `%1` is used once (in `%5`).
`%2` is used once (in `%4`). But if there were more consumers, they would all
reference the same `MIRValueId`. For example, `x + x` loads `x` once and the DAG
has one `Load(x)` node with two consumers.

This flat representation is how LLVM IR, Cranelift IR, and QBE IL all work.
It is the standard for compiler IRs because it makes optimization passes
trivial: constant folding replaces `%id` with the constant value ID,
common subexpression elimination merges two identical instruction-producing
IDs into one, dead code elimination removes IDs with zero consumers.

### 2.2 Instruction definition

```rust
/// A single MIR instruction. Produces exactly one value, identified by `id`.
/// The `id` is globally unique within the function.
#[derive(Debug, Clone)]
pub struct MIRInstruction {
    pub id: MIRValueId,
    pub kind: MIRInstructionKind,
    /// The concrete type of the value this instruction produces.
    /// Every instruction must know its result type.
    pub result_type: MIRTypeId,
}

#[derive(Debug, Clone)]
pub enum MIRInstructionKind {
    // ── Constants (zero operands, produce a literal value) ────────────────

    /// Integer constant. Width is determined by result_type (Word or Long).
    /// For Word, the constant is implicitly truncated to 32 bits.
    /// For Long, the full 64 bits are used.
    ///
    /// QBE: `%x =w copy 42` or `%x =l copy 42`
    /// LLVM: implicit constant — no instruction needed, just use `i32 42`
    ConstInt(u64),

    /// Floating-point constant.
    ///
    /// QBE: `%x =s copy 3.14` or `%x =d copy 3.14`
    ConstFloat(f64),

    /// The unit value `()`. Produces a zero-sized marker.
    /// Codegen emits nothing for this.
    ///
    /// QBE: no QBE equivalent — just skip
    ConstUnit,

    // ── Memory operations (explicit alloca / load / store) ───────────────

    /// Allocate stack space for a value of the given type.
    /// Returns a pointer to the allocated space.
    ///
    /// In QBE: `%p =l alloc4 1` (allocates 4 bytes, 1 slot, returns a long pointer)
    /// In LLVM: `%p = alloca i32`
    ///
    /// Allocas are placed in the function's entry block (like LLVM's
    /// mem2reg convention) so they dominate all uses.
    ///
    /// The HIR variable `x := 42` lowers to:
    ///   %px = alloca Word          ; allocate stack slot for x
    ///   %v  = ConstInt(42)          ; the value
    ///   store %px, %v               ; write it to x's slot
    Alloca(MIRTypeId),

    /// Store a value to a memory location.
    ///
    /// `ptr` is a pointer value (from Alloca, FieldPtr, or an array index).
    /// `value` is the value to write. The store width is determined by
    /// the type of `value`.
    ///
    /// QBE: `storew %val, %ptr`  (store word)
    ///       `storel %val, %ptr`  (store long)
    ///       `stores %val, %ptr`  (store single-precision float)
    ///       `stored %val, %ptr`  (store double-precision float)
    ///
    /// Store produces `Unit` (it is a statement, not an expression).
    Store { ptr: MIRValueId, value: MIRValueId },

    /// Load a value from a memory location.
    ///
    /// The ptr must be a pointer value. The loaded value's type is
    /// determined by the MIRType's pointee type (for Pointer) or by
    /// the element type (for array indexing).
    ///
    /// QBE: `%x =w loadw %ptr`  (load word)
    ///       `%x =l loadl %ptr`  (load long)
    ///
    /// For small integers (u8, i8, etc.) stored as Word, the load must
    /// be followed by a truncation or sign extension. These are emitted
    /// as separate instructions (Trunc, ZExt, SExt) after the load.
    ///
    /// This explicit load/store model matches LLVM's alloca + mem2reg
    /// pipeline. A later "mem2reg" pass can promote allocas that are
    /// only written once per block and never have their address taken
    /// into MIRValueId-based SSA, eliminating the alloca/load/store
    /// triplet entirely.
    Load { ptr: MIRValueId },

    // ── Integer arithmetic ───────────────────────────────────────────────

    /// Addition. Both operands must be same width (both Word or both Long).
    /// Result width equals operand width.
    ///
    /// QBE: `%x =w add %a, %b`  or  `%x =l add %a, %b`
    Add(MIRValueId, MIRValueId),

    /// Subtraction. Same rules as Add.
    ///
    /// QBE: `%x =w sub %a, %b`  or  `%x =l sub %a, %b`
    Sub(MIRValueId, MIRValueId),

    /// Multiplication. Same rules as Add.
    ///
    /// QBE: `%x =w mul %a, %b`  or  `%x =l mul %a, %b`
    Mul(MIRValueId, MIRValueId),

    /// Signed division. Both operands must be same signed integer type.
    ///
    /// QBE: `%x =w div %a, %b`  or  `%x =l div %a, %b`
    /// Note: QBE's `div` is signed division. For unsigned, use `UDiv`.
    SDiv(MIRValueId, MIRValueId),

    /// Unsigned division. Same as SDiv but uses QBE's `udiv` instruction.
    ///
    /// QBE: `%x =w udiv %a, %b`  or  `%x =l udiv %a, %b`
    UDiv(MIRValueId, MIRValueId),

    /// Signed remainder (modulo). Result has same sign as dividend.
    ///
    /// QBE: `%x =w rem %a, %b`  or  `%x =l rem %a, %b`
    SRem(MIRValueId, MIRValueId),

    /// Unsigned remainder.
    ///
    /// QBE: `%x =w urem %a, %b`  or  `%x =l urem %a, %b`
    URem(MIRValueId, MIRValueId),

    // ── Floating-point arithmetic ────────────────────────────────────────

    FAdd(MIRValueId, MIRValueId),
    FSub(MIRValueId, MIRValueId),
    FMul(MIRValueId, MIRValueId),
    FDiv(MIRValueId, MIRValueId),

    // ── Bitwise operations ───────────────────────────────────────────────

    /// Bitwise AND.
    ///
    /// QBE: `%x =w and %a, %b`  or  `%x =l and %a, %b`
    And(MIRValueId, MIRValueId),

    /// Bitwise OR.
    ///
    /// QBE: `%x =w or %a, %b`  or  `%x =l or %a, %b`
    Or(MIRValueId, MIRValueId),

    /// Bitwise XOR.
    ///
    /// QBE: `%x =w xor %a, %b`  or  `%x =l xor %a, %b`
    Xor(MIRValueId, MIRValueId),

    /// Left shift.
    ///
    /// QBE: `%x =w shl %a, %b`  or  `%x =l shl %a, %b`
    Shl(MIRValueId, MIRValueId),

    /// Logical right shift (zero-fill).
    ///
    /// QBE: `%x =w shr %a, %b`  or  `%x =l shr %a, %b`
    Shr(MIRValueId, MIRValueId),

    /// Arithmetic right shift (sign-fill).
    ///
    /// QBE: `%x =w sar %a, %b`  or  `%x =l sar %a, %b`
    Sar(MIRValueId, MIRValueId),

    // ── Integer comparisons (produce Word: 0 or 1) ───────────────────────

    /// Equality comparison.
    ///
    /// QBE: `%x =w ceqw %a, %b`  or  `%x =w ceql %a, %b`
    CmpEq(MIRValueId, MIRValueId),

    /// Not-equal comparison.
    ///
    /// QBE: `%x =w cnew %a, %b`  or  `%x =w cnel %a, %b`
    CmpNe(MIRValueId, MIRValueId),

    /// Signed less-than.
    ///
    /// QBE: `%x =w csltw %a, %b`  or  `%x =w csltl %a, %b`
    CmpSLt(MIRValueId, MIRValueId),

    /// Signed less-than-or-equal.
    ///
    /// QBE: `%x =w cslew %a, %b`  or  `%x =w cslel %a, %b`
    CmpSLe(MIRValueId, MIRValueId),

    /// Signed greater-than.
    ///
    /// QBE: `%x =w csgtw %a, %b`  or  `%x =w csgtl %a, %b`
    CmpSGt(MIRValueId, MIRValueId),

    /// Signed greater-than-or-equal.
    ///
    /// QBE: `%x =w csgew %a, %b`  or  `%x =w csgel %a, %b`
    CmpSGe(MIRValueId, MIRValueId),

    /// Unsigned less-than.
    ///
    /// QBE: `%x =w cultw %a, %b`  or  `%x =w cultl %a, %b`
    CmpULt(MIRValueId, MIRValueId),

    /// Unsigned less-than-or-equal.
    ///
    /// QBE: `%x =w culew %a, %b`  or  `%x =w culel %a, %b`
    CmpULe(MIRValueId, MIRValueId),

    /// Unsigned greater-than.
    ///
    /// QBE: `%x =w cugtw %a, %b`  or  `%x =w cugtl %a, %b`
    CmpUGt(MIRValueId, MIRValueId),

    /// Unsigned greater-than-or-equal.
    ///
    /// QBE: `%x =w cugew %a, %b`  or  `%x =w cugel %a, %b`
    CmpUGe(MIRValueId, MIRValueId),

    // ── Float comparisons (produce Word: 0 or 1) ─────────────────────────

    CmpFEq(MIRValueId, MIRValueId),
    CmpFNe(MIRValueId, MIRValueId),
    CmpFLt(MIRValueId, MIRValueId),
    CmpFLe(MIRValueId, MIRValueId),
    CmpFGt(MIRValueId, MIRValueId),
    CmpFGe(MIRValueId, MIRValueId),

    // ── Type conversions ─────────────────────────────────────────────────

    /// Zero-extend a smaller integer value to a wider type.
    ///
    /// Example: loading a u8 into a Word.
    ///   %raw = Load { ptr: %alloca_u8 }
    ///   %w   = ZExt { value: %raw, to: WordTypeId }
    ///
    /// QBE: `%w =w extub %raw`  (extend unsigned byte)
    ///       `%w =w extuh %raw`  (extend unsigned halfword, 16 bits)
    ZExt { value: MIRValueId, to: MIRTypeId },

    /// Sign-extend a smaller signed integer value to a wider type.
    ///
    /// Example: loading an i8 into a Word.
    ///   %raw = Load { ptr: %alloca_i8 }
    ///   %w   = SExt { value: %raw, to: WordTypeId }
    ///
    /// QBE: `%w =w extsb %raw`  (extend signed byte)
    ///       `%w =w extsh %raw`  (extend signed halfword, 16 bits)
    SExt { value: MIRValueId, to: MIRTypeId },

    /// Truncate a wider integer to a narrower one.
    ///
    /// Example: storing a Word into a u8 slot.
    ///   %trunc = Trunc { value: %word_val, to: U8TypeId }
    ///   Store { ptr: %alloca_u8, value: %trunc }
    ///
    /// QBE: no direct trunc instruction. Truncation is implicit in QBE's
    ///       `storew`/`storel` — the bottom N bytes of the word/long are
    ///       written. For our MIR, we keep Trunc as an explicit instruction
    ///       because LLVM needs `trunc i32 %x to i8`. In QBE codegen, this
    ///       lowers to a no-op — the store emits the right width.
    Trunc { value: MIRValueId, to: MIRTypeId },

    /// Convert a signed integer to a float.
    ///
    /// QBE: `%f =s sitof %i`  or  `%f =d sitof %i`
    SIToF { value: MIRValueId, to: MIRTypeId },

    /// Convert a float to a signed integer, truncating toward zero.
    ///
    /// QBE: `%i =w ftosi %f`  or  `%i =l ftosi %f`
    FToSI { value: MIRValueId, to: MIRTypeId },

    // ── Aggregate operations ──────────────────────────────────────────────

    /// Extract a field value from an aggregate (struct or tuple).
    ///
    /// `base` is the value holding the aggregate. `field` is the zero-based
    /// field index (computed from the HIR's string-based member name during
    /// lowering — the HIR has `MemberAccess(expr, "field_name")`, and we
    /// resolve which index "field_name" corresponds to in the struct layout).
    ///
    /// This is for extracting from a value, not a pointer. If you have a
    /// pointer to a struct and want field access, use FieldPtr + Load.
    ///
    /// QBE: QBE has no struct type, so structs are never values produced by
    ///       instructions — they live entirely in memory. Struct field access
    ///       is always: FieldPtr(base_ptr, field_idx) → Load(ptr).
    ///       However, tuples returned from functions may use ExtractValue.
    ExtractValue { base: MIRValueId, field: usize },

    /// Get a pointer to a field within an aggregate.
    ///
    /// `base` is a pointer to the start of the struct/array. `field` is the
    /// zero-based field index. The instruction computes:
    ///
    ///   new_ptr = base_ptr + field_offset[field]
    ///
    /// This is the MIR equivalent of LLVM's `getelementptr`.
    ///
    /// QBE: `%fptr =l add %base, offset`
    ///       (the offset is a constant computed at lowering time)
    FieldPtr { base: MIRValueId, field: usize },

    // ── Function calls ────────────────────────────────────────────────────

    /// Call a function.
    ///
    /// `callee` is the function name (for now — a MIRFuncId once we intern
    /// function references). `args` are the MIRValueIds of the arguments.
    ///
    /// A function call that produces a value has result_type = the return type.
    /// A function call that returns Unit has result_type = Unit.
    ///
    /// QBE: `%result =w call $funcname(%arg1, %arg2, ...)`
    ///       (or `call $funcname(...)` for void / unit returns)
    Call { callee: String, args: Vec<MIRValueId> },

    // ── SSA Phi node ──────────────────────────────────────────────────────

    /// Phi instruction: selects a value based on which predecessor block
    /// control flow came from.
    ///
    /// Each `(value, block_id)` pair means "if we arrived at this phi's block
    /// from `block_id`, then this phi produces `value`".
    ///
    /// Phi nodes must appear at the start of a block (before any other
    /// instructions). All phi nodes in a block execute simultaneously
    /// (conceptually) — they all read values from the predecessor blocks
    /// before any assignments happen in the current block.
    ///
    /// **Example** (if-then-else assigning to y):
    ///
    ///   entry:
    ///     %cond = ...   ; some condition
    ///     CondBr %cond → then, else
    ///
    ///   then:
    ///     %y1 = ConstInt(1)
    ///     Jump merge
    ///
    ///   else:
    ///     %y2 = ConstInt(2)
    ///     Jump merge
    ///
    ///   merge:
    ///     %y = Phi { incoming: [(%y1, then), (%y2, else)] }
    ///     ; use %y ...
    ///
    /// QBE: `%y =w phi %y1 @then, %y2 @else`
    /// LLVM: `%y = phi i32 [ %y1, %then ], [ %y2, %else ]`
    ///
    /// **Why block parameters (Cranelift-style) are not used here:**
    ///
    /// Cranelift uses "extended basic blocks" (EBBs) where jump instructions
    /// pass arguments that bind to the target block's parameters. This
    /// elegantly models blocks-as-expressions without phi nodes:
    ///
    ///   then:
    ///     %y1 = ConstInt(1)
    ///     Jump merge(%y1)     ; pass %y1 as the block parameter
    ///
    ///   else:
    ///     %y2 = ConstInt(2)
    ///     Jump merge(%y2)
    ///
    ///   merge(%y):             ; block parameter %y receives the jumped value
    ///     ; use %y ...
    ///
    /// While elegant, this model is **not supported by QBE or LLVM**.
    /// Both use phi nodes. Since our primary target is QBE (and LLVM as
    /// a secondary option), we use phi nodes for a 1:1 lowering.
    /// Additionally, phi nodes are the standard in the SSA literature and
    /// in essentially every production compiler except Cranelift.
    Phi {
        /// Pairs of (value_produced, predecessor_block_that_produced_it).
        /// The order matches the predecessor order for determinism.
        incoming: Vec<(MIRValueId, MIRBlockId)>,
    },
}
```

### 2.3 Why logical ops (&&, ||) are lowered to branches + phi, not instructions

The HIR has `BinOp::LogAnd`, `BinOp::LogOr`, `BinOp::LogXor`. In C-like
languages, `&&` and `||` have **short-circuit semantics**:

```
a && b  →  if !a then false else b
a || b  →  if a then true else b
```

The right-hand side is NOT evaluated if the left-hand side determines the
result. This is fundamentally **control flow**, not arithmetic. The MIR has
no `And`/`Or` instructions for booleans — these are lowered to branches.

Additionally, `^^` (logical XOR) does NOT have short-circuit semantics
(both sides must be evaluated). However, we keep the lowering to instructions
consistent:

```
^^` is lowered to: %a xor %b  // bitwise XOR of two Word values (0 or 1 each)
This produces 0 if a == b and 1 if a != b, which is exactly logical XOR.
No branches needed for `^^`.
```

**Lowering `a && b`:**

```
// Zea: result := a && b;
//
// HIR:
//   HIRExpression {
//     kind: BinOpExpr(LogAnd,
//       HIRExpression { kind: ScopedIdent("a") },
//       HIRExpression { kind: ScopedIdent("b") })
//   }
//
// MIR (word-sized bools):
//
// block_entry:
//   %0 = Load { ptr: %alloca_a }
//   %1 = CmpNe(%0, ConstInt(0))       ; test a != 0 (truthiness)
//   CondBr { cond: %1, true_block: eval_b, false_block: short_circuit }
//
// eval_b:
//   %2 = Load { ptr: %alloca_b }
//   %3 = CmpNe(%2, ConstInt(0))       ; test b != 0 (truthiness)
//   Jump merge
//
// short_circuit:
//   // No need to load b — we short-circuited
//   Jump merge
//
// merge:
//   %result = Phi {
//     incoming: [(%3, eval_b), (ConstInt(0), short_circuit)]
//   }
//   Store { ptr: %alloca_result, value: %result }
```

**Lowering `a || b`:**

```
// Zea: result := a || b;
//
// block_entry:
//   %0 = Load { ptr: %alloca_a }
//   %1 = CmpNe(%0, ConstInt(0))
//   CondBr { cond: %1, true_block: short_circuit, false_block: eval_b }
//
// eval_b:
//   %2 = Load { ptr: %alloca_b }
//   %3 = CmpNe(%2, ConstInt(0))
//   Jump merge
//
// short_circuit:
//   Jump merge
//
// merge:
//   %result = Phi {
//     incoming: [(%3, eval_b), (ConstInt(1), short_circuit)]
//   }
//   Store { ptr: %alloca_result, value: %result }
```

**Lowering `a ^^ b` (no short-circuit):**

```
// Zea: result := a ^^ b;
//
// block_entry:
//   %0 = Load { ptr: %alloca_a }
//   %1 = Load { ptr: %alloca_b }
//   %result = Xor(%0, %1)            ; bitwise XOR — both are word-sized 0/1
//   Store { ptr: %alloca_result, value: %result }
```

**Unary `!a` (logical not):**

```
// Zea: result := !a;
//
// block_entry:
//   %0 = Load { ptr: %alloca_a }
//   %1 = CmpEq(%0, ConstInt(0))      ; a == 0 → true (1), otherwise false (0)
//   Store { ptr: %alloca_result, value: %1 }
```

This is why the MIR has no `Not` instruction — logical not is just `CmpEq(x, 0)`.

The HIR's `UnOp::LogNot` and `UnOp::BitNot` become:
- `LogNot(e)` → `CmpEq(e_val, ConstInt(0))`
- `BitNot(e)` → use `Xor(e_val, ConstInt(all_ones))` or a dedicated `Not`
  instruction added later as an optimization. For now, BitNot is `Xor(x, -1)`.
  (QBE does not have a `not` instruction — you XOR with -1.)

### 2.4 HIR operator → MIR instruction mapping

| HIR BinOp   | MIR Instruction(s)       | Notes |
|-------------|--------------------------|-------|
| `Add`       | `Add` or `FAdd`          | Integer or float, based on operand type |
| `Sub`       | `Sub` or `FSub`          | |
| `Mul`       | `Mul` or `FMul`          | |
| `Div`       | `SDiv` or `UDiv`         | Choice based on whether operands are signed |
| `Mod`       | `SRem` or `URem`         | |
| `LogAnd`    | Lowered to branches+phi  | Short-circuit control flow |
| `LogOr`     | Lowered to branches+phi  | Short-circuit control flow |
| `LogXor`    | `Xor`                    | No short-circuit; both sides evaluated |
| `BitAnd`    | `And`                    | |
| `BitOr`     | `Or`                     | |
| `BitXor`    | `Xor`                    | |
| `Subscript` | FieldPtr + Load          | Array indexing: index * elem_size + base, then load |
| `Lsh`       | `Shl`                    | |
| `Rsh`       | `Shr` or `Sar`           | Choice based on operand signedness |
| `Eq`        | `CmpEq` or `CmpFEq`      | Integer or float, based on operand type |
| `Neq`       | `CmpNe` or `CmpFNe`      | |
| `Geq`       | `CmpSGe`/`CmpUGe`/`CmpFGe` | |
| `Leq`       | `CmpSLe`/`CmpULe`/`CmpFLe` | |
| `LT`        | `CmpSLt`/`CmpULt`/`CmpFLt` | |
| `GT`        | `CmpSGt`/`CmpUGt`/`CmpFGt` | |

| HIR UnOp    | MIR Instruction(s)        | Notes |
|-------------|---------------------------|-------|
| `Neg`       | `Sub(ConstInt(0), val)` or `FSub`   | Integer: 0 - x. Float: specific FNeg or FSub(0.0, x) |
| `LogNot`    | `CmpEq(val, ConstInt(0))` | !x means x == 0 |
| `BitNot`    | `Xor(val, ConstInt(-1))`  | ~x means x XOR all-ones |

---

## Part 3 — Basic Blocks, Terminators & Functions

### 3.1 What is a basic block?

A basic block is a straight-line sequence of instructions with no internal
branches. It has exactly one entry point (the first instruction) and exactly one
exit point (the terminator). Control flow always enters at the top and leaves
through the terminator.

In the CFG (control-flow graph), basic blocks are nodes and terminators are the
directed edges.

Most compiler IRs use basic blocks: LLVM, Cranelift, QBE, Java bytecode,
WebAssembly. It is the standard way to represent structured and unstructured
control flow.

### 3.2 Phi nodes explained (from first principles)

A phi node solves the problem of **single static assignment (SSA)** at control
flow merge points. In SSA form, each variable is assigned exactly once. But at
a merge point (after an if-then-else), a variable might have been assigned
different values in different branches:

```
if x > 0 {
    y = 1;    // y gets value 1 ← but which y?
} else {
    y = 2;    // y gets value 2 ← which y?
}
print(y);     // ← we need to use ONE y here
```

In a non-SSA IR, `y` is just a memory slot — you store 1 or 2 to it, then load
it at the print. But in SSA, you cannot assign to `%y` twice. The phi node
resolves this:

```
entry:
    %cond = CmpSGt(%x, ConstInt(0))
    CondBr %cond → then, else

then:
    %y1 = ConstInt(1)
    Jump merge

else:
    %y2 = ConstInt(2)
    Jump merge

merge:
    %y = Phi { incoming: [(%y1, then), (%y2, else)] }
    Call { callee: "print", args: [%y] }
```

The phi does not "execute" — it is a **notational device**. The semantics are:
when the processor reaches `merge`, the register/file holding `%y` already
contains either `%y1` (if we came from `then`) or `%y2` (if we came from
`else`). In practice, the register allocator resolves phis by assigning `%y`,
`%y1`, and `%y2` to the **same physical register**, so the "selection" is
automatic — the branch that ran already put the right value in that register.

QBE places phi nodes at the top of blocks:

```
@merge
    %y =w phi %y1 @then, %y2 @else
    call $print(w %y)
```

LLVM does the same:

```
merge:
    %y = phi i32 [ %y1, %then ], [ %y2, %else ]
    call void @print(i32 %y)
```

### 3.3 MIR basic block structure

```rust
/// A basic block in MIR.
///
/// Contains a sequence of instructions and exactly one terminator.
/// Phi instructions (if any) must appear first in the instruction list,
/// before any non-phi instructions. All phis in a block execute
/// simultaneously (they read values from predecessor blocks).
#[derive(Debug, Clone)]
pub struct MIRBasicBlock {
    pub id: MIRBlockId,
    /// Instructions in execution order. Phi nodes first, then arithmetic.
    /// Terminator is separate, not part of this list.
    pub instructions: Vec<MIRInstruction>,
    /// The terminator — exactly one, always present.
    pub terminator: MIRTerminator,
}

/// Control flow transfer out of a basic block.
///
/// Every block must end with exactly one terminator.
/// The terminator determines which block executes next.
#[derive(Debug, Clone)]
pub enum MIRTerminator {
    /// Unconditional jump to another block.
    ///
    /// QBE: `jmp @target`
    /// LLVM: `br label %target`
    Jump { target: MIRBlockId },

    /// Conditional branch.
    ///
    /// `cond` must be a Word-sized value (the result of a comparison).
    /// If cond != 0, control goes to `true_block`. Otherwise, `false_block`.
    ///
    /// QBE: `jnz %cond, @true_block, @false_block`
    /// LLVM: `br i1 %cond, label %true_block, label %false_block`
    ///
    /// QBE also has `jnp` (jump if pointer, i.e. non-null) but we don't use
    /// that — all conditions are word-sized comparisons.
    CondBr {
        cond: MIRValueId,
        true_block: MIRBlockId,
        false_block: MIRBlockId,
    },

    /// Return from the function.
    ///
    /// `value` is the return value. If `None`, the function returns `Unit`
    /// (void in C terms).
    ///
    /// QBE: `ret %val`  or  `ret`  (for void/unit returns)
    /// LLVM: `ret i32 %val`  or  `ret void`
    Return { value: Option<MIRValueId> },

    /// Unreachable / divergent terminator.
    ///
    /// Inserted after calls to functions that never return (e.g., `exit`,
    /// `panic`), or in dead code paths.
    ///
    /// QBE: `hlt`  (halt — causes undefined behavior if reached at runtime)
    /// LLVM: `unreachable`
    Unreachable,
}
```

### 3.4 MIR function structure

```rust
/// A MIR function: a set of basic blocks connected by terminators.
///
/// The function body is a control-flow graph. `entry` is the id of the first
/// block to execute. All blocks must be reachable from `entry` (post-lowering
/// cleanup enforces this).
#[derive(Debug, Clone)]
pub struct MIRFunction {
    /// The function's name, matching the HIR function name.
    pub name: String,

    /// Parameter value IDs. Function arguments get MIRValueIds so they can be
    /// referenced as operands by instructions in the entry block. In LLVM and
    /// QBE, function arguments are implicitly the first "values" in the entry
    /// block — we make this explicit by assigning them MIRValueIds.
    pub params: Vec<MIRValueId>,

    /// Human-readable parameter names (debugging / printing).
    pub param_names: Vec<String>,

    /// MIR types of each parameter (same order as `params`).
    pub param_types: Vec<MIRTypeId>,

    /// Return type of the function.
    pub return_type: MIRTypeId,

    /// The entry block — where execution starts.
    pub entry: MIRBlockId,

    /// All basic blocks in this function, indexed by MIRBlockId.
    /// Using IndexMap preserves insertion order for deterministic output,
    /// while still providing O(1) lookup by ID.
    pub blocks: IndexMap<MIRBlockId, MIRBasicBlock>,

    /// All instructions across all blocks, indexed by MIRValueId.
    /// This is the flat-value arena — the core of the DAG representation.
    /// Every MIRInstruction.id maps to its definition here.
    ///
    /// Using a separate arena (rather than storing instructions inside blocks
    /// by value) means multiple blocks can reference the same MIRValueId.
    /// For example, a constant `ConstInt(0)` can be defined once and
    /// referenced by loads, comparisons, and stores across the entire
    /// function.
    pub values: IndexMap<MIRValueId, MIRInstruction>,
}
```

### 3.5 The flat value arena (DAG in practice)

This separation — `blocks` contains instruction lists, `values` is a flat
IndexMap of all instructions — is the key design decision. It enables:

1. **DAG sharing.** Two `Load` instructions can share the same `Alloca` ID.
   Three `CmpNe` checks can share the same `ConstInt(0)`. A `ConstInt(0)` used
   in 50 places still has one definition.

2. **Use-def chains.** Given a `MIRValueId`, you can look up its definition
   (the instruction that produced it) in O(1) from `values`. Given the
   definition, you can walk its operands: `Add(%lhs, %rhs)` gives you two more
   MIRValueIds to look up. This is the standard way to walk a DAG.

3. **Dead code elimination.** Count uses of each MIRValueId. Any instruction
   with zero uses (that is not a side-effecting instruction like Store or Call)
   can be removed. The phi nodes in blocks can have their operand counts
   checked — if a phi has one incoming edge left, it's an identity and can be
   folded.

4. **Common subexpression elimination.** Hash instructions by
   (kind, operands, result_type). When you're about to emit a new instruction,
   check if an identical one already exists. If so, reuse its MIRValueId instead
   of emitting a duplicate.

5. **Constant folding.** If all operands of an instruction are constants
   (ConstInt, ConstFloat), evaluate the result at compile time and replace
   the instruction's MIRValueId with the folded constant's MIRValueId.

### 3.6 HIR → MIR function lowering example

Consider this Zea function:

```
fn max(a: i32, b: i32) -> i32 {
    if a > b {
        return a;
    } else {
        return b;
    }
}
```

The HIR for this would be:

```
HIRFunction {
    name: "max",
    params: [a: i32, b: i32],
    returns: i32,
    body: HIRBlockExpression {
        statements: [],
        last: HIRExpression {
            kind: IfThenElse(HIRBranch {
                condition: BinOpExpr(GT, ScopedIdent("a"), ScopedIdent("b")),
                true_case: Block({ Return(ScopedIdent("a")) }),
                false_case: Block({ Return(ScopedIdent("b")) }),
            })
        }
    }
}
```

The MIR after lowering would be:

```
Function: max(a: w, b: w) -> w

Block entry:
  %p_a  = Alloca(Word)       ; stack slot for parameter a
  %p_b  = Alloca(Word)       ; stack slot for parameter b
  Store { ptr: %p_a, value: %a }
  Store { ptr: %p_b, value: %b }
  %v_a  = Load { ptr: %p_a } ; read a
  %v_b  = Load { ptr: %p_b } ; read b
  %cond = CmpSGt(%v_a, %v_b)  ; a > b ?
  CondBr { cond: %cond, true_block: then, false_block: else }

Block then:
  %ret_a = Load { ptr: %p_a }
  Return { value: %ret_a }

Block else:
  %ret_b = Load { ptr: %p_b }
  Return { value: %ret_b }
```

(Note: the allocas and stores for params can be eliminated by a later mem2reg
pass — the params %a and %b are already MIRValueIds and can be used directly.)

---

## Part 4 — MIR Module (Top-Level)

```rust
/// A complete compilation unit in MIR form.
///
/// Contains all types, global variables, and functions.
/// This is the output of HIR→MIR lowering and the input to codegen.
#[derive(Debug, Clone)]
pub struct MIRModule {
    /// Links back to the originating HIR module. Retained for error reporting
    /// and debugging — the HIR node ID can be used to trace MIR constructs
    /// back to source locations.
    pub id: NodeId,

    /// All concrete types in the module, interned and indexed by MIRTypeId.
    /// This is a flat table — every MIRInstruction's result_type field is a
    /// key into this map. Types are deduplicated: two `Word` types share the
    /// same MIRTypeId.
    pub types: IndexMap<MIRTypeId, MIRType>,

    /// Top-level variable initializations (Zealand: `x: i32 = 42;` at module
    /// scope). These become global variables in QBE/LLVM.
    pub globals: Vec<MIRGlobal>,

    /// All functions defined in this module.
    pub functions: Vec<MIRFunction>,
}

/// A module-level variable with an initializer.
///
/// In QBE, emitted as: `data $name = { w 42 }`
/// In LLVM: `@name = global i32 42`
#[derive(Debug, Clone)]
pub struct MIRGlobal {
    /// The variable name, as in source.
    pub name: String,

    /// The type of the global variable.
    pub typ: MIRTypeId,

    /// The initializer value. Must be a constant (ConstInt, ConstFloat, etc.)
    /// — no loads, calls, or allocas allowed. The HIR→MIR lowering pass
    /// must verify that global initializers are compile-time constants.
    pub init: MIRValueId,
}
```

### 4.1 Global initialization

In Zea, module-level initializations can use simple literals and arithmetic on
other globals:

```
module main
x: i32 = 42;
y: i32 = x + 1;
```

The HIR→MIR lowerer must compute `x + 1` at compile time (constant folding
during lowering) to produce a single `ConstInt(43)` for `y`. This is the same
mechanism as C's constant expressions in global initializers.

---

## Part 5 — HIR → MIR Lowering Pipeline

### 5.1 Overview

The HIR→MIR lowering pass takes a type-checked, scope-resolved `HIRModule` and
produces a `MIRModule`. This is the step where the tree becomes a DAG, the
implicit becomes explicit, and the high-level becomes backend-ready.

### 5.2 Steps

1. **Type translation.** Walk all `HIRTypeSpecifier`s in the HIR and create
   corresponding `MIRType` entries in the MIR type table. Compute sizes and
   alignments. For structs, run the layout algorithm (field reordering for
   minimal padding) and compute field offsets.

2. **Global lowering.** For each global `HIRInitializationBlock` in the
   HIR module, produce a `MIRGlobal`. Fold the initializer expression to a
   constant value. If the initializer references other globals, resolve those
   references.

3. **Function lowering.** For each `HIRFunction`:
   a. Create the function skeleton: params, return type, empty block/values maps.
   b. Create the entry block. Emit `Alloca` instructions for every local
      variable and parameter.
   c. Walk the function body (a `HIRBlockExpression`). For each statement
      and expression, emit MIR instructions.
   d. For control flow (if-then-else, return), emit basic blocks with
      appropriate terminators and phi nodes.

4. **Expression lowering** — the core of the tree→DAG transformation:

   For each `HIRExpression`, produce a `MIRValueId`:
   - **Literal** → `ConstInt` / `ConstFloat` / `ConstUnit` instruction.
   - **ScopedIdent** → `Load` from the variable's `Alloca` (or direct use of
     the value in mem2reg-eligible cases).
   - **BinOp** → Recursively lower lhs and rhs to MIRValueIds, then emit the
     appropriate arithmetic/comparison instruction.
   - **UnOp** → Recursively lower operand, then emit the appropriate instruction
     (or for LogNot: `CmpEq(val, ConstInt(0))`).
   - **LogAnd / LogOr** → Create the short-circuit branch structure (entry block,
     eval block, merge block with phi).
   - **IfThenElse** → Create three blocks: a condition block (evaluates cond
     and branches), a then-block, and an else-block. Both join at a merge block
     with a phi if the if-expression produces a value.
   - **Block (block-as-expression)** → A new MIR basic block. Emit statements
     as instructions, emit the tail expression, and the block's result is the
     value produced by the tail.
   - **FunctionCall** → Lower all arguments to MIRValueIds, emit `Call`
     instruction.
   - **MemberAccess** → Lower base to a pointer (or alloca). Look up the field
     index in the struct layout. Emit `FieldPtr` + `Load`.

5. **Mem2reg pass (optional, later optimization).** Promote allocas that are
   only ever written once per definition and never have their address taken.
   Replace `Alloca + Store + Load` chains with direct MIRValueId references.
   This is the same algorithm LLVM's `-mem2reg` pass uses (based on the SSA
   construction algorithm by Cytron et al., 1991).

### 5.3 Full example: lowering a block-as-expression

Zea source:
```
module main

add_one: u8 = 1;

fn main() -> u8 {
    b := { let x: u8 = add_one; x + 1 };
    return b;
}
```

HIR (simplified, after scoping and typechecking):

```
HIRModule {
  globals: [
    HIRInitializationBlock {
      kind: Unpacked([
        HIRSimpleInitialization {
          assignee: "add_one",
          typ: Some(u8),
          value: IntegerLiteral(1)
        }
      ])
    }
  ],
  functions: [
    HIRFunction {
      name: "main",
      returns: u8,
      body: HIRBlockExpression {
        statements: [
          Initialization(Unpacked([
            HIRSimpleInitialization {
              assignee: "b",
              typ: Some(u8),
              value: Block(HIRBlockExpression {  // <-- block-as-expression
                statements: [
                  Initialization(Unpacked([
                    HIRSimpleInitialization {
                      assignee: "x",
                      typ: Some(u8),
                      value: ScopedIdent(add_one)  // load global
                    }
                  ]))
                ],
                last: BinOpExpr(Add, ScopedIdent(x), IntegerLiteral(1))
              })
            }
          ]))
        ],
        last: Return(ScopedIdent(b))
      }
    }
  ]
}
```

MIR (after lowering):

```
MIRModule {
  types: [
    MIRType { id: 0, kind: Word, size: 4, alignment: 4 },
    MIRType { id: 1, kind: Word, size: 4, alignment: 4 },  // dupe? dedup'd
  ],
  globals: [
    MIRGlobal { name: "add_one", typ: 0, init: %g0 }
  ],
  functions: [
    MIRFunction {
      name: "main",
      params: [], param_names: [], param_types: [],
      return_type: 0,  // Word (u8 promoted)
      entry: block0,
      blocks: {
        block0: MIRBasicBlock {
          id: block0,
          instructions: [
            %0  = Alloca(Word)                                    ; alloca for b
            %1  = Alloca(Word)                                    ; alloca for x
          ],
          terminator: Jump { target: block1 }
        },

        block1: MIRBasicBlock {                       ; block { let x = ...; x + 1 }
          id: block1,
          instructions: [
            %g0 = Load { ptr: global_ptr_add_one }              ; load global add_one
            Store { ptr: %1, value: %g0 }                       ; x = add_one
            %x  = Load { ptr: %1 }                              ; read x
            %2  = ConstInt(1)
            %3  = Add(%x, %2)                                   ; x + 1
            Store { ptr: %0, value: %3 }                        ; b = block result
          ],
          terminator: Jump { target: block2 }
        },

        block2: MIRBasicBlock {
          id: block2,
          instructions: [
            %b = Load { ptr: %0 }                                ; read b
          ],
          terminator: Return { value: %b }
        }
      },
      values: {
        %0:  Alloca(Word),
        %1:  Alloca(Word),
        %g0: Load { ptr: ... },
        %x:  Load { ptr: %1 },
        %2:  ConstInt(1),
        %3:  Add(%x, %2),
        %b:  Load { ptr: %0 },
      }
    }
  ]
}
```

(After mem2reg, the Alloca/Store/Load for `x` and `b` would be eliminated and
replaced with direct value references.)

---

## Part 6 — QBE Lowering Reference

### 6.1 QBE instruction set (relevant subset)

QBE has a minimal, orthogonal instruction set. The suffix determines the type:

| Suffix | Type | Width | Description |
|--------|------|-------|-------------|
| `w` | word | 32 bits | Integer, pointer, boolean |
| `l` | long | 64 bits | Large integer, pointer on 64-bit |
| `s` | single | 32 bits | IEEE 754 single-precision float |
| `d` | double | 64 bits | IEEE 754 double-precision float |

**Arithmetic:**

| QBE | Description |
|-----|-------------|
| `%x =w add %a, %b` | Integer addition |
| `%x =w sub %a, %b` | Integer subtraction |
| `%x =w mul %a, %b` | Integer multiplication |
| `%x =w div %a, %b` | Signed division |
| `%x =w udiv %a, %b` | Unsigned division |
| `%x =w rem %a, %b` | Signed remainder |
| `%x =w urem %a, %b` | Unsigned remainder |

**Bitwise:**

| QBE | Description |
|-----|-------------|
| `%x =w and %a, %b` | Bitwise AND |
| `%x =w or %a, %b` | Bitwise OR |
| `%x =w xor %a, %b` | Bitwise XOR |
| `%x =w shl %a, %b` | Left shift |
| `%x =w shr %a, %b` | Logical right shift (zero-fill) |
| `%x =w sar %a, %b` | Arithmetic right shift (sign-fill) |

**Comparisons (produce `w` result, 0 or 1):**

| QBE | Description |
|-----|-------------|
| `%x =w ceqw %a, %b` | Equal |
| `%x =w cnew %a, %b` | Not equal |
| `%x =w csltw %a, %b` | Signed less-than |
| `%x =w cslew %a, %b` | Signed less-than-or-equal |
| `%x =w csgtw %a, %b` | Signed greater-than |
| `%x =w csgew %a, %b` | Signed greater-than-or-equal |
| `%x =w cultw %a, %b` | Unsigned less-than |
| `%x =w culew %a, %b` | Unsigned less-than-or-equal |
| `%x =w cugtw %a, %b` | Unsigned greater-than |
| `%x =w cugew %a, %b` | Unsigned greater-than-or-equal |

**Float arithmetic (same names, different suffix):**

| QBE | Description |
|-----|-------------|
| `%x =s add %a, %b` | Float addition (single) |
| `%x =d add %a, %b` | Float addition (double) |
| ... | (same for sub, mul, div — no udiv/rem for floats) |

**Float comparisons (produce `w` result, 0 or 1):**

| QBE | Description |
|-----|-------------|
| `%x =w ceqs %a, %b` | Single equality |
| `%x =w cnes %a, %b` | Single not-equal |
| `%x =w clts %a, %b` | Single less-than |
| `%x =w cles %a, %b` | Single less-or-equal |
| `%x =w cgts %a, %b` | Single greater-than |
| `%x =w cges %a, %b` | Single greater-or-equal |
| `%x =w ceqd %a, %b` | Double equality |
| ... | (same pattern with `d` suffix) |

**Memory:**

| QBE | Description |
|-----|-------------|
| `%p =l alloc4 N` | Allocate N 4-byte slots, return pointer |
| `%p =l alloc8 N` | Allocate N 8-byte slots |
| `%p =l alloc16 N` | Allocate N 16-byte slots |
| `%x =w loadw %p` | Load a word from pointer |
| `%x =l loadl %p` | Load a long from pointer |
| `%x =s loads %p` | Load a single from pointer |
| `%x =d loadd %p` | Load a double from pointer |
| `storew %x, %p` | Store a word to pointer |
| `storel %x, %p` | Store a long to pointer |
| `stores %x, %p` | Store a single to pointer |
| `stored %x, %p` | Store a double to pointer |

**Extensions/truncations:**

| QBE | Description |
|-----|-------------|
| `%w =w extub %b` | Extend unsigned byte (8→32 bits) |
| `%w =w extuh %h` | Extend unsigned halfword (16→32 bits) |
| `%w =w extsb %b` | Extend signed byte |
| `%w =w extsh %h` | Extend signed halfword |
| `%l =l extsw %w` | Extend signed word to long (32→64) |
| `%l =l extuw %w` | Extend unsigned word to long |

**Integer ↔ Float conversion:**

| QBE | Description |
|-----|-------------|
| `%f =s sitof %i` | Signed int to single float |
| `%f =d sitof %i` | Signed int to double float |
| `%f =s uitof %i` | Unsigned int to single float |
| `%f =d uitof %i` | Unsigned int to double float |
| `%i =w ftosi %f` | Float to signed int (truncate toward 0) |
| `%i =l ftosi %f` | Double to signed long |

**Control flow:**

| QBE | Description |
|-----|-------------|
| `jmp @block` | Unconditional jump |
| `jnz %cond, @true, @false` | Jump if cond != 0 |
| `ret %val` | Return value |
| `ret` | Return void |
| `hlt` | Unreachable / halt |
| `call $func(...)` | Function call |

**Phi:**

| QBE | Description |
|-----|-------------|
| `%x =w phi %a @blockA, %b @blockB` | Phi node (word type) |

### 6.2 MIR instruction → QBE lowering (exhaustive)

For each MIR instruction kind, the direct QBE output:

| MIR Instruction | QBE Output | Notes |
|---|---|---|
| `ConstInt(v)` | `%x =w copy v` or `%x =l copy v` | Type suffix from result_type |
| `ConstFloat(v)` | `%x =s copy v` or `%x =d copy v` | |
| `ConstUnit` | *(nothing)* | No QBE value produced |
| `Alloca(t)` | `%p =l allocN 1` | N = MIRType.size of `t` |
| `Store { ptr, value }` | `storew %value, %ptr` | Suffix from value's type |
| `Load { ptr }` | `%x =w loadw %ptr` | Suffix from result_type |
| `Add(l, r)` | `%x =w add %l, %r` | Suffix from result_type |
| `Sub(l, r)` | `%x =w sub %l, %r` | |
| `Mul(l, r)` | `%x =w mul %l, %r` | |
| `SDiv(l, r)` | `%x =w div %l, %r` | |
| `UDiv(l, r)` | `%x =w udiv %l, %r` | |
| `SRem(l, r)` | `%x =w rem %l, %r` | |
| `URem(l, r)` | `%x =w urem %l, %r` | |
| `FAdd(l, r)` | `%x =s add %l, %r` | Suffix from result_type |
| *(all float ops)* | *(same pattern)* | |
| `And(l, r)` | `%x =w and %l, %r` | |
| `Or(l, r)` | `%x =w or %l, %r` | |
| `Xor(l, r)` | `%x =w xor %l, %r` | |
| `Shl(l, r)` | `%x =w shl %l, %r` | |
| `Shr(l, r)` | `%x =w shr %l, %r` | |
| `Sar(l, r)` | `%x =w sar %l, %r` | |
| `CmpEq(l, r)` | `%x =w ceqw %l, %r` | `l` suffix for Long |
| `CmpSLt(l, r)` | `%x =w csltw %l, %r` | |
| `CmpULt(l, r)` | `%x =w cultw %l, %r` | |
| *(all cmps)* | *(same pattern)* | |
| `CmpFEq(l, r)` | `%x =w ceqs %l, %r` | `d` suffix for Double |
| `ZExt { value, to }` | `%w =w extub %v` or `extuh` or `extuw` | Depends on source width |
| `SExt { value, to }` | `%w =w extsb %v` or `extsh` or `extsw` | |
| `Trunc { value, to }` | *(no-op)* | QBE stores truncate implicitly |
| `SIToF { value, to }` | `%f =s sitof %v` | `d` suffix for Double |
| `FToSI { value, to }` | `%i =w ftosi %v` | `l` for Long result |
| `ExtractValue { base, field }` | *(decompose)* | No direct QBE equivalent |
| `FieldPtr { base, field }` | `%fp =l add %base, offset` | Offset is compile-time constant |
| `Call { callee, args }` | `%r =w call $callee(%a1, %a2, ...)` | Type suffix from return type |
| `Phi { incoming }` | `%x =w phi %v1 @blk1, %v2 @blk2, ...` | |

---

## Part 7 — Structure Summary

The complete MIR node hierarchy:

```
MIRModule
├── id: NodeId
├── types: IndexMap<MIRTypeId, MIRType>
│       ├── MIRTypeId → MIRType { id, kind, size, alignment }
│       └── MIRTypeKind: Unit | Word | Long | Single | Double
│                        | Pointer(MIRTypeId)
│                        | Array { elem, count }
│                        | Struct { name, fields }
├── globals: Vec<MIRGlobal>
│       └── MIRGlobal { name, typ: MIRTypeId, init: MIRValueId }
└── functions: Vec<MIRFunction>
        ├── name: String
        ├── params: Vec<MIRValueId>
        ├── param_names: Vec<String>
        ├── param_types: Vec<MIRTypeId>
        ├── return_type: MIRTypeId
        ├── entry: MIRBlockId
        ├── blocks: IndexMap<MIRBlockId, MIRBasicBlock>
        │       ├── MIRBlockId → MIRBasicBlock { id, instructions, terminator }
        │       └── MIRTerminator: Jump | CondBr | Return | Unreachable
        └── values: IndexMap<MIRValueId, MIRInstruction>
                └── MIRValueId → MIRInstruction { id, kind, result_type }
                     └── MIRInstructionKind:
                          ConstInt | ConstFloat | ConstUnit
                          Alloca | Load | Store
                          Add | Sub | Mul | SDiv | UDiv | SRem | URem
                          FAdd | FSub | FMul | FDiv
                          And | Or | Xor | Shl | Shr | Sar
                          CmpEq | CmpNe | CmpSLt | CmpSLe | CmpSGt | CmpSGe
                          CmpULt | CmpULe | CmpUGt | CmpUGe
                          CmpFEq | CmpFNe | CmpFLt | CmpFLe | CmpFGt | CmpFGe
                          ZExt | SExt | Trunc | SIToF | FToSI
                          ExtractValue | FieldPtr
                          Call | Phi
```
