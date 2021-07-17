---
title: Disk format
---

There is no filesystem as we usually know them, but data is still stored on the disk in a given format.

All numbers are represented as big endian.

At the begining of the partition, there are some headers. First of all, there should be a magic number: `0x0ADB`.
Then there are three `u16`, indicating the version of the database format (major, minor and patch respectively).

The database is divided in blocks. All blocks have the same size. Each block contains one type of data.

After the version number comes the number of blocks in the database and the size of one block (in bytes),
both represented as `u64`.

Then comes the type table. Actually there is two copies of it, preceeded by a checksum of one of these table.
In practice, the checksum is not implemented yet, but 64 bytes are zeroed.

A type table is just a series of `u64`, one for each block, corresponding to the ID of the type
stored in the corresponding block. Some types are garanteed to have a given ID:

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

Very small and schematic example of a type table :

```
0x4
0xc
0xc
0xd
0xc
```

It starts with the size of the type table : `0x4` = `4`. Then we know that the two first sectors will store `Type`s,
and then we have one `Function` sector, and then another `Type` sector.

## Block format

Each block starts with the number of items in the block (`u64`).
Then comes the space that is already used in this block (`u64` too), including "headers".

Then comes a list of "pointers", that are indices (`u64`) pointing to a byte
from the start of the block. Each of these pointers points to an item
of the block.

Starting from the end of the block, the actual items are stored.

## Polymorphism

Only monomorphized types are stored: `Type` doesn't have parameters.

Polymorphic type definitions are stored as functions that only have static
parameters and return a `Type`.

## Non-trivial type definitions

### Type / U

```rust
struct Type {
    name : String
    id: TypeId,
    definition: enum {
        Sum {
            variants : Map<String, TypeId>,
        },
        Product {
            fields : Map<String, TypeId>,
        },
        Array {
            of: TypeId,
        },
    },
}
```

