# Specification

This more a "let's dump all my ideas in one file" than a specification,
but yeah.

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

## Precise Specification

### Type system

We define a few natives types, embedded in the "compiler":

- `!`, that cannot be built
- `()`, that can be built in exactly one way: `()`
- `bool`, that is either `true` or `false`
- Combinations of `u`/`i` and `8`/`16`/`32`/`64`/`128`/`size`, for integers
- `f32`/`f64` for floats
- `char` for a single Unicode code point (the C char is `u8`)

And we introduce the following rules to build new types:

#### Sum types

You define a list of constructors, with eventual associated data.
If unambiguous, the constructors may be anonymous.

```rust
type Foo = Bar | Baz
type X = Y(i32) | Z(f32)
type A = i32 | f32
```

#### Product types

You define a set of values to have to get a value of your type.
Each value may or may not have a name.

```rust
type Foo = Bar * Baz
type X = y : i32 * z : f32
type A = x : X * Foo 
```

#### Dependent types

There is two kind of values that you can work on: compile-time values,
and run-time values. This allows us to have dependent types.
The first kind of values is written between `<>` to make a difference.

```
type SizedVec<size : usize * t : Type> = ...
```

The above type will get monomorphized at compile time. This equivalent
will not, and will use run-time polymorphism:

```
type SizedVec(size : usize * t : Type) = ...
```

Thus, we can see types as functions returning something of type `Type`.

All of a function compile-time parameters get monomorphized, and you'll
have as much variants of the same function as needed in your final binary.
This allows the developer to choose where to put the balance between
performance and binary size.

If a function only has compile-time arguments, it may be called at compile-time.
It may make the type system turing complete, making inference impossible in some
cases, but maybe we could limit control structures to run time to limit this issue?
I think we will see depending if we really need inference, or if adding types
annotations is not too inconvenient.

#### Traits

Traits are basically functions `Type -> Type` saying if a type matches
some constraints. It returns `!` if it does not, and the same type if
it does. This way, we can do:

```rust
trait X { ... }
fn foo<x : Type * impl_x : X(x)>(arg : impl_x) { ... }
```

Because compile-time expressions are not compiled, just run in some kind
of VM/interpreter by the compiler.

#### Modules

You can split code in modules, that can themselves contain modules, and you can import
modules. Pretty much like JS, Python, Rust, or any modern language.

#### Pointers


### Package publishing

- Central package repo? Distributed self-hostable repos?
- Rules before publication:
  - Every function should be documented
  - Every breaking change in types should include a migration

### Disk format

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

### The `Type` type

Because this type would need itself to be defined it is a "virtual" type that the OS always knows about,
without needing to read its definition.

```rust
enum Type {
    qual_name: String,
    type_parameters: Vec<>,
    data: Vec<Type>, // variants for sum types, fields for product types
}
```


## API

Whats is need is :

- A way to iterate over a stream of data
- A way to save a stream of data

If the saved stream of data is a stream of references, the
objects that are referenced are updated. If they are "owned"
values, they are added to the DB.

Syscalls (high-level version?): 

```rust
open_stream<T>(): Stream<T>; // creates a stream of objects of type `type` in the DB
next_of_stream<T>(&Stream<T>): Option<T>; // iterates over a stream
save_stream<T>(&Stream<T>); // saves a stream
close_stream<T>(Stream<T>); // closes a stream
```

Some kind of programming language is also needed. To have an unique format
once again, features could be enabled and disabled per-package. For instance,
you could turn off borrow-checking and move semantics and have everything
boxed for you when needed (not sure if it is actually possible??).

## Virtual objects in the DB

Some objects should be accessible via the DB but without actually being on disk.
Hardware could be abstracted that way: a monitor would just be a regular bitmap
for instance. The system time could also be acceded that way.

To set a pixel on the screen all you would have to do would be change a value on
"the disk" (actually nothing would be written).

## Permissions

Each object in the database (or database page for better performances? that
would force us to group object of the same type-permissions combination together tho,
fragmenting the disk a bit more) could have an associated `u64`, pointing to a
specific function saying if an user-package combination is authorized to access this object or not.

```rust
fn simple_access<T>(user: &User, package: &Package) -> bool {
    user.name == "Alice" && package.permissions.types.includes<T>()
}
```

## Reative UI

A UI framework should come with the system. Its architecture should
probably be inpired by React+Redux or Elm: data goes from top to bottom,
events go from bottom to top.

Each component could be a `Future<Event>` that gets polled for new events regularly.

The DE would be the only app to have access to `InputEvent`s in the DB. It would then
compute which "window" should receive the event, and pass it down. It would then be the window's
job to pass the event to the appropriate child, and so on, until the event is handle by a child in the
hierarchy.

## Signaling

Drivers and apps may want to subscribe to modifications to the database, how to do it?

## Environments: communication with the outside world

An environment is a way to abstract a database. Your computer is an environment,
a USB is another one, a web server is another one, etc. This allows for conversion
of data format between environments (big endian to little endian, wrapping data in TCP packets, etc).

An environment is defined as :

```rust
trait Environment {
    fn open_stream<T>(&self) -> Stream<Result<T, IOError>>;
    fn save_stream<T>(&self, Stream<T>) -> Future<Result<(), IOError>>;
}
```

Then, drivers and packages can provide concrete types, that apps can use.

## Reasons performances may be better

- Not years of legacy that has to be maintained
- No time spent on parsing and serializing data: you can just copy from RAM to disk directly.
- Because we have one and only one database, if it is very well optimized, all apps will benefit from it (on traditional system, each app has to optimize their DB)

## Inspiration

- <http://blog.rfox.eu/en/Programmer_s_critique_of_missing_structure_of_oper.html/>
- <http://www.righto.com/2017/10/the-xerox-alto-smalltalk-and-rewriting.html?showComment=1508781022450#c7613952874348706529>
- <http://okmij.org/ftp/papers/DreamOSPaper.html>
- <http://zge.us.to/txt/unix-harmful.html>
- <https://github.com/aluntzer/gtknodes>

And also probably (just bookmarks, I have not read them yet):

- <https://eighty-twenty.org/2016/05/05/unix-is-incorrectly-factored>
- <https://programmingmadecomplicated.wordpress.com/2017/08/12/there-is-only-one-os-and-its-been-obsolete-for-decades/>
- <http://www.vpri.org/pdf/tr2007008_steps.pdf>
- <http://witheve.com/>

## Similar projects

- <https://en.wikipedia.org/wiki/WinFS>
- <http://bitsavers.trailing-edge.com/pdf/symbolics/software/genera_8/Genera_Concepts.pdf>
- IBM i
- Smalltalk machines
- NewtonOS?
- <http://phantomos.org/>
- <https://en.wikipedia.org/wiki/Singularity_(operating_system)>
- Plan9?
- BeOS?
- Haiku?
