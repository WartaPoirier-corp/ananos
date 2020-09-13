---
title: Executable format
---

Because our programs use structured data as input and outputs, we cannot use PE or ELF as an executable format.

So we need to come up with our own format. Here is a proposal.

```rust
struct Version(u16, u16, u16);

struct VirtualAddress(usize);

struct ExecutablePackage {
    name: String,
    version: Version,
    dependencies: Vec<(VirtualAddress, &ExecutablePackage)>,
    code: Vec<u8>,
}
```

When starting this executable, the OS loads each dependency at the requested address, and then loads the code and jumps to it.

The first function of the code takes it's arguments on the stack, and returns a value by putting it on the stack and calling the `exit` system call.

## Potential issues

What if we have this dependency tree?

- A
  - B
  - C
    - B

C and A will need to load B at two different addresses, while it could be shared easily.

TODO: look at position independant code, it might solve this issue.
