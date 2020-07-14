---
title: Drivers
---

Drivers are functions of type `() -> !` (they can be started without any additional
info and run forever). If they need additional information, they can get it by reading
data exposed by the kernel/other drivers in the shared memory. They expose their own interface
in the shared memory as well.

There should a standard trait for each kind of driver one may encounter. For instance,
a `NetworkCard` trait with functions to get the MAC address, read a packet and write one.

Drivers might run in userspace or in kernel, as they are just functions.
