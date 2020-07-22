---
title: Disk format
---

There is no filesystem as we usually know them, but data is still stored on the disk in a given format.
There should be a type table at the very begining of the disk, saying which sectors stores which type.
A type is uniquely identified by a `u64`, with these "special" cases:

- TODO : add basic types like `i`/`u`/`f`-`8`/`16`/`32`/`64`/`128`/`size` ;
- `0x0` is the `Type` type ;
- `0x1` is the `Function` type ;
- TODO : add `Any` (that just has reflection and nothing else) ;

The type table is of the size `size_of_disk / number_of_sector` so that each sector can have a given type stored into it.
This size is stored as the very first `u64` of the table
Each entry is just a `u64` indicating what type is stored in the sector having the same index as the current type table entry.

Very small and schematic example of a type table :

```
0x100
0x0
0x0
0x1
0x0
```

It starts with the size of the type table : `0x100` = `4`. Then we know that the two first sectors will store `Type`s,
and then we have one `Function` sector, and then another `Type` sector.

