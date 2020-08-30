---
title: Disk format
---

There is no filesystem as we usually know them, but data is still stored on the disk in a given format.
There should be a type table at the very begining of the disk, saying which sectors stores which type.
A type is uniquely identified by a `u64`, with these "special" cases:

- `0x0`: `!`
- `0x1`: `()`
- `0x2`: `u8`
- `0x3`: `u16`
- `0x4`: `u32`
- `0x5`: `u64`
- `0x6` to `0x9`: `i8` to `i64`
- `0xa`: `f32`
- `0xb`: `f64`
- `0xc` is the `Type` type (aka `U`) ;
- `0xd` is the `Function` type ;
- `0xe` is the `Trait` type ;

The type table is of the size `size_of_disk / number_of_sector` so that each sector can have a given type stored into it.
This size is stored as the very first `u64` of the table
Each entry is just a `u64` indicating what type is stored in the sector having the same index as the current type table entry.

Very small and schematic example of a type table :

```
0x100
0xc
0xc
0xd
0xc
```

It starts with the size of the type table : `0x100` = `4`. Then we know that the two first sectors will store `Type`s,
and then we have one `Function` sector, and then another `Type` sector.

## Polymorphism

Only monomorphized types are stored: `Type` doesn't have parameters.

Polymorphic type definitions are stored as functions that only have static
parameters and return a `Type`.

## Non-trivial type definitions

### Type / U

```rust
let Type = name : String
         * (
             sum : (
                 variants : Map<String, Type>,
             )
             +
             product : (
                 fields : Map<String, Type>,
             )
         )
```

### Function

```rust
let Assembly = Vec<u8> // I guess??

let Function = name : String
             * static_args : Type
             * dynamic_args : Type
             * return_type : Type
             * body : Assembly
```
