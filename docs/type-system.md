---
title: Type system
---

**TODO:** is type equivalence really necessary, and more importantly:
is SAFE?

## TL;DR

- A few base types (numbers)
- Algebraic types
- Dependent types

## The base types

There a few base types, that more or less correspond to
what a CPU can natively handle.

- `u` means unsigned integer
- `i` means signed integer
- `f` means float

And after comes the size in bits.

The basic types are: `u8`, `u16`,`u32`, `u64`,
`i8`, `i16`, `i32`, `i64`, `f32`, `f64`.

There is also `usize` and `isize` which take
32 bits on 32 bits architectures and 64 bits
on 64 bits architectures.

**TODO:** Do we add 128 bits types?

## Sum types

### Introduction rule

Given two types `A` and `B`, the type `A + B`
is the sum type of A and B, that is a type that either
contains a value of type `A` or a value of type `B`.

To make it easier to know which variant is contained,
we associate a name with each case (that will be a byte
in memory).

We can generalize this contruction with as many types as
we want, giving us sum types of any number of cases, by
saying that `B` itself is a sum of two types `C` and `D` (and so on).

### Elimination rule

To eliminate (that is, to access the value contained in this type)
we have to give a way to handle each of its case (that is usually
called pattern-matching). Each case handler can safely access the wrapped
value, because it is only executed if the type of the value is indeed the
one we expect.

`A + B`, `A → C` and `B → C` can be used to obtain `C`.

### « Empty » or « Never » type

There is also a special case of sum types, that is the
sum type with zero variants, called « empty » type
or « never » type (because a value of this type can never
be built).

This type will be written as `!` from now on.

If `A` is a type, `A + !` can be considered equivalent to `A`.

## Product types

### Introduction rule

We also define a product type. Let `A` and `B` be two
types. Then `A * B` is the types of values that are made
of a value of type `A` and a value of type `B`.

Using the same pattern as for sum types, we can generalize that
to obtain product types of any sizes.

Each element that composes such a type may be named, for easier
identification and referencement later.

### Elimination rule

Products can be eliminated by destructuring. We can choose
to consider only one of the member independently of the other.
We can also consider each member independently without discarding
any of them.

`A * B` can be used to obtain `A`, or to obtain `B`.

### « Unit » type

Similarly to `!`, we define a special case of product types,
the unit type, that is the product type with exactly zero
elements.

This type is written as `()` from now on.

If `A` is a type `A * ()` can be considered equivalent to `A`.

## Type algebra

- `A * (B + C) ~ (A + B) * (A + C)`

## Function types

Let `A` and `B` be two types. `A → B` is the type of
the functions that associates a `A` to a `B`.

### Introduction rule

Functions are introduced by an expression of type `B`,
that may use a value of type `A`.

**TODO:** find a better way to express that, I don't like the above
sentence.

### Elimination rule

Functions can be eliminated by *applying* them. Given a `A → B` and
a `A`, we can obtain a `B`, replacing the `A` in the definition of the function
by the value that was provided.

## Dependent types

Types may also depend on values from other types.

A type `A` that depends on a value `b : B` will be written `A = <b : B → U>`.
If there are more than one of them, they are separated by a `→`, because
we will use currying to handle multiple type arguments, allowing
partially-applied types.

These values themselves may depend on previous « type parameters ».
For instance, we can have a type `A = <b : B → c : C<b> → U>`.

### Universes

We introduce a type `U` (for *Universe*) that is the type
of all other types (but not itself).

**TODO:** do we need U0, U1, …, Un, Un+1 universes?

### Compile-time versus run-time

Dependent parameters are computed by the compiler at *compile-time*.
Function parameters (the `A` in `A → B`) are computed at *run-time*.

You can thus choose between dynamic and static arguments, and thus
dynamic or static polymorphism very easily.

These two functions have equivalent types, but one can be computed
at compile-time, while the other will need to be compiled and then run:

```
compiled_func : <B : U → b : B → A>
runtime_func :   B : U → b : B → A
```

**TODO:** This allows for functions that creates types at run-time, is
it a good idea? Maybe `U` shouldn't be available at run-time?

## Constraints

Types can thus be seen as functions that are computed at compile-time,
returning values of type `U`.

Constraints are compile-time functions of type `U → (() + ())` (or `U → bool`, at a
higher-level). The constraint returns the first variant (`true`) if the
type is respected, and the second one (`false`) if it is not the case.
The type-checker can thus be extended: if all the constraints on a type
are respected, it is well-typed, otherwise there is a type error.

We can use this system to build traits (AKA type classes).

## References

**TODO**

## Exemples of types

```
// Partial type application
Map = <K : U → V : U → U>
List<T : U> = Map<usize>
IntList = List<i32>
OtherIntList = Map<usize><i32>

// Traits
MyTrait = <Type : U → trait_func : (Type → A → B) → (() + ())>
MyType = ()
a_to_b_for_my_type : MyType → A → B
function_that_needs_my_trait : <T : U, MyTrait<T, a_to_b_for_my_type>> (x : T → ())
```
