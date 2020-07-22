---
title: System calls
---

At a very high level (like we use async and everything, but
in reality we will get a pointer to a Waker and that kind of things).

## Memory management

- `malloc(size) -> Result<&Any, ()>`
- `free(ptr: &Any)`

## Database interaction

- `open<T: Type>(T, limit: u64) -> StreamHandle<T>`, limit of 0 means as much as possible
- `async read(StreamHandle<T>, location: Option<LocationId>) -> T`
- `async write(StreamHandle<T>, T) -> WriteStatus`
- `close(StreamHandle<_>)`

Almost all other traditional system calls can be emulated with the database:
reading from a device, getting system time, opening a TCP connection, etc.

---

```rust
enum WriteStatus {
    /// In case the location (RAM, HDD, SSD, USB key, server, etc) was unambiguous
    Done,
    /// In case it was not clear where to do the write
    UnknownLocation,
}

impl WriteStatus {
    async fn define_location_if_needed(self, location: LocationId) -> WriteStatus {}
}

struct LocationId(u64);
```
