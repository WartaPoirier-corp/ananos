---
title: System calls
---

At a very high level (like we use async and everything, but
in reality we will get a pointer to a Waker and that kind of things).

## Memory management

- `malloc(size) -> Result<&Any, ()>`
- `free(ptr: &Any)`

## Database interaction

Database interaction is build around *streams*, that can be though as
file descriptors for structured data.

- `open(T : U * limit : u64) -> StreamHandle<T>`
   Limit of 0 means as much as possible
- `async read(T : U * stream : StreamHandle<T> * location : Option<LocationId>) -> T`
   Reads the next object of the stream (similarly to `Iterator::next` in Rust).
   If location is specified only objects from this location will be accepted.
   Otherwise, objects can any location may be returned.
- `async write(T : U * stream : StreamHandle<T> * location : Option<LocationId> * obj : T) -> Result<T * IoError>`
   Write an object to a given stream. This adds it to the end of the stream, and
   can eventually have side effects depending on the location (like writing to the disk,
   or sending data to a server). If `location` is not specified, it will be saved to
   the default location (usually the RAM).

Once done with a stream, it should be `free`-ed.

Almost all other traditional system calls can be emulated with the database:
reading from a device, getting system time, opening a TCP connection, etc.

