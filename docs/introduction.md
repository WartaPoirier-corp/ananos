---
title: Introduction
---

## Issues with current OSes

Computers are basically made for manipulating data.
You load it, apply changes to it, display it, and eventually save it.

But current systems add to much abstractions over this simple idea, ironically for the sake of "simplification".
With every new layer of "simplification" actually comes more complexity, because it is harded to understand what the computer is doing for real.

Some metaphors that are currently used are also outdated and only add complexity.
For instance, file systems were used as a metaphor of real world files.
But now that most people left paper for computers, it isn't the best way to organize data anymore.
The metaphor doesn't speak to people anymore.

The idea from the UNIX philosophy that all data on your computer "is a file", could just become all data "is data".
We don't need to have different formats for storing the same kind of information.
We don't need multiple implementations of the same methods for manipulating these information.

## General ideas

That's why I believe that a new kind of OS could be built, bringing a new approach to computing.
The most importants ideas are :

- There is no filesystem.
  Programs can ask the OS to store and load structured data directly.
  The OS acts as a common database.
- This allows us to follow the UNIX principle saying "do one thing and do it well", and to bring it further.
  Programs can take structured data as input and output other structured data.
- This means that there is a need for a common type system, and common standard types.
  The OS should come with basic types for everything that computer usually do:
  from integers, arrays, and strings to images, videos, and so on.
- A graphical interface to "pipe" data from one program to another could be built,
  allowing people to build complex logic for manipulating their data.
  If this interface provides a way to define new types, use basic logic blocks and make system calls,
  and save a "pipeline" as a standalone program, there would be no need for different programming languages.
- Communication between devices could be abstracted as "pulling" data from, and "pushing" data to remote "databases".
  Both networking and external storage devices could be abstracted this way.
- Drivers could be standard programs.
  For each kind of driver a common interface could be defined, and every program that has this interface could be used as a driver.


