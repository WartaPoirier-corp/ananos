---
title: Executable format
---

Because our programs use structured data as input and outputs, we cannot use PE or ELF as an executable format.

## Overview

Programs are stored as bytecode, and the kernel JITs it before loading it in memory.

This bytecode guarantees that no "illegal"/"dangerous" operation can be made. Thus, it allows us to run user programs in ring 3, and why not all in the same address space, which improves performance (no need to flush and rebuild the TLB constantly) and makes it easier to pass memory objects around (no need to check that pointers are correct for both programs, and so on).

A program is just a single function. Each program can have dependencies on other functions that the OS will load at the same time as the program.

The bytecode contains placeholders for function addresses, and type identifiers (used in system calls at least). The JIT will need to replace them with actual addresses and identifiers.

System calls could also be implemented as regular functions that are "linked" with user programs, instead of using `int 0x80` or `sysenter`, or anything else.

There will be no difference between threads and processes, since all programs share the same memory anyway. A function can however ask the OS to spawn another function in parallel, similarly to threads on other OSes.

Using bytecode instead of assembly also makes it easier to compile/JIT for other platforms (with a "bridge" for ananOS system calls).

## What is needed

- A specification of the high-level language
- A bytecode specification
- A memory representation specification
- An executable format specification
- A bytecode compiler (that can run on POSIX machines, but why not on ananOS too)
- A JIT-compiler, that can run in ananOS

Being able to run in ananOS means `#![no_std] + extern crate alloc` (no access to FS APIs and co., but Vec/BTreeMap/Arc/etc are available).

## Specification of the language

The actual syntax is yet to be determined, but it should have:

- Basic module system : 
    - Similar to Rust: one file = one module, module hierarchy follows FS hierarchy
    - No need for public/private for the moment IMO
- Type system:
    - Basic types: u{8, 16, 32, 64}, i{8, 16, 32, 64}, f{32, 64}
    - Algebraic data types
        - Sum types (aka enums)
        - Product types (aka structs)
        - Never (`!`) and Unit (`()`)
    - Special "Array" type: a `u64` defining a length, and the content of the array (the type of the items is known because this type actually gets monomorphized: `[u8]` and `[u16]` don't have the same type IDs)
- Functional:
    - All values are immutable (even if the resulting assembly code may actually mutate them for optimization purposes)
    - Functions have one input and one output (but it may be product types). No currying for the moment IMO, it would make things more complicated than needed.
- Functions that does I/O should be marked as such (either with a monad à la Haskell, or with a keyword like `async` or `io`)
    - This will allow for a lot of optimization
- Move semantics:
    - Any value passed to another function cannot be used anymore
    - If you want to "borrow" it, you have to return it along with the actual result of the function
    - The compiler could optimize this pattern as pointers OFC
    - For instance, this code: `fn f(x: &T) -> U;` would be written as `fn f(x: T) -> (U, T)`
- Error handling: there will at least be Option/Result, not sure if panicking is needed. Maybe if we go for an IO monad, it could also be used to track errors?
- A Core module to interact with the OS (the specification will be written once the syntax of the language is defined).

## Bytecode specification

Here too, not much is set in stone yet.

- Some instructions are "privileged" (they will be reserved to drivers), and the JIT must error if a non-privileged function tries to use them
- The bytecode must garantee that no illegal memory access is possible, either statically or at runtime (with runtime bound-checks if they couldn't be checked at compile time for instance).
- It should contain placeholders for type IDs and functions that will be linked when doing the final compilation

## Memory representation

Because the OS will have to copy data from the DB to apps memory, the memory representation must be coherent.

### Basic types

Numeric types take the space they usually take. They are in big endian for integer types. Floats use the usual IEEE-???? representation.

Arrays use `u64` for their length.

Never and Unit are zero-sized.

### Sum types

The discriminant/tag is encoded as a `u64`. The associated data, if any, is following. The space reserved for associated data is as big as it can be (`i.e` for `() + u64` it will be 64 bits long, even if the value represents the first variant).

### Product types

Each field is laid out next to each other in the defined order.

This means that the compiler can't reorder fields for optimization, but it may suggest the user to do it.

TODO: alignment + padding? (help welcome)

## Executable format specification

Executables are regular database objects, with the following type:

```rust
struct Executable {
    name: String,
    input_type: TypeId,
    output_type: TypeId,
    dependencies: Map<Placeholder, ExecutableName>,
    types: Map<Placeholder, TypeName>,
    bytecode: Bytecode,
}

// type defs, not sure about them, but just to give an idea
type Bytecode = [u8];
type Placeholder = u64;
type ExecutableName = String;
type TypeName = String;
```

When compiling bytecode to assembly, the compiler will find the actual functions and types needed, load functions in memory (recursively, because they may have dependencies as well), and replace placeholders with actual pointers to the functions and type IDs.

System calls will be regular dependencies of executables, `int 0x80` or other classic system call mechanism won't be used.

