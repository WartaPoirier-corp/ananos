---
title: Virtual Database
---

At the core of this OS is a database of structured data, instead of files.

Similarly to Linux that exposes some files that are not really on any disk,
the database will primarly be a virtual database.

It will agregate data from the RAM, the disk(s) and eventually
USB keys, CDs, or even remote servers.

A `Location` type will be provided to list and choose an appropriate location
when writing data (thanks to functions like `find_main_location`, `find_memory_location`, etc.).


