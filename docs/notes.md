---
title: Unfinished notes
---

### Package publishing

- Central package repo? Distributed self-hostable repos?
- Rules before publication:
  - Every function should be documented
  - Every breaking change in types should include a migration

## Programming

Some kind of programming language is also needed. To have an unique format
once again, features could be enabled and disabled per-package. For instance,
you could turn off borrow-checking and move semantics and have everything
boxed for you when needed (not sure if it is actually possible??).

The [core types](core-types.md) should include an AST that could be manipulated directly if
given an appropriate UI. So no real "syntax" to define or anything like that, which is nice.

## Debugging

Automatically save debug info and allow to inspect and/or send them to devs when there is a crash/bug/issue.

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

Drivers and apps may want to subscribe to modifications to the database. An idea might
be to do something like:

```ocaml
let on_new_picture (img : Picture) : Widget =
  debug_display img

let main =
  let sub = get_subscriber_from_db in
  sub#subscribe (typeof Picture) EventKind.Creation on_new_picture
```

And then have a driver call all the functions registered this way when something changes in the DB.

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

## Reasons performance may be worse

- I don't know how to write optimized code
- I don't know how to design an efficient database
- I don't know how to write an OS

But I'm learning about all of that, and other people with more experience may help as well, who knows.

## Things to explore / questions that are not yet resolved

- Actor model, to avoid threads and mutex, and why not the need for a borrow checker (see the Pony language)
- Drawing code instead of typing (see [RAND Grail](https://en.wikipedia.org/wiki/RAND_Tablet))

## Inspiration

- <http://blog.rfox.eu/en/Programmer_s_critique_of_missing_structure_of_oper.html/>
- <http://www.righto.com/2017/10/the-xerox-alto-smalltalk-and-rewriting.html?showComment=1508781022450#c7613952874348706529>
- <http://okmij.org/ftp/papers/DreamOSPaper.html>
- <http://zge.us.to/txt/unix-harmful.html>
- <https://github.com/aluntzer/gtknodes>
- <https://eighty-twenty.org/2016/05/05/unix-is-incorrectly-factored>
- <https://eighty-twenty.org/2011/05/08/weaknesses-of-smalltalk-strengths-of-erlang>
- <https://www.youtube.com/watch?v=NY6XqmMm4YA&feature=youtu.be&t=585>
- <https://www.youtube.com/watch?v=8pTEmbeENF4>
- <https://programmingmadecomplicated.wordpress.com/2017/08/12/there-is-only-one-os-and-its-been-obsolete-for-decades/>

And also probably (just bookmarks, I have not read them yet):

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

