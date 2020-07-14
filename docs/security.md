---
title: Security
---

Security is defined on two or three level:

- read
- write
- and optionnaly, execute, for functions

By default, everything is allowed for everyone.

Types can define their own security policy by implementing a `Security` trait:

```rust
trait Security {
    /// Can this process read anything about this type
    ///
    /// If this is false, the process won't even be able to
    /// make reflexion about this type, not even see it exists.
    fn can_see(process: Process) -> bool;

    /// Can this process read this element from the DB?
    fn can_read(&self, process: Process) -> bool;

    /// Can this process write this new element
    /// to the DB?
    fn can_write(&self, process: Process) -> bool;
}

struct Process {
    signature: Type,
    parent: &Process,
}
```

A common use case will be to check which user started this process.
The user session process will depend on a `user : User` parameter
that will be accessible in the signature of this process, which can
be accessed from any process just by going up in the chain of parent
process.

There is an additional `ExecutionSecurity` trait for callable objects:

```
trait ExecutionSecurity {
    fn can_run(&self, process: Process) -> bool;
}
```

Here too, if not specified, `can_run = true`.

All of these functions should be deterministic, so that their result can be cached (for performance).
The fact that they are not `async` will make it very difficult to query the database
for additional info, so we should be safe anyway (and because the kernel will assume they are
deterministic, if they are not, the behavior will be wrong anyway).
