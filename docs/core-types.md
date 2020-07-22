---
title: Core types
---

Types that should be in the standard library.

## Basic types

```
/// Async and failible I/O operations
Io<Result : U, Error : U>

/// Operation that can fail
Result<Ok : U, Error : U>

/// Zero or one value
Option<Type : U>
```

## Needed for the system to work as expected by most people

```
/// A user identity
Identity;

/// A user session
///
/// When a new identity is created we monomorphize a new Session type.
Session<id : Identity>
```

Some package management system.

## Other core modules

Modules that should be defined by the OS, but are related to a specific domain.

- Text (with formating)
- Image
- Audio
- Video
- Discussion
- An AST (to create your own functions and that kind of things)
- Some DOM/web browser API? Complex and should be avoided ideally
- A webcam/camera API
- Subscriptions (for news feed, etc)

- paramètres
- code
- internet
- compta
- news
- caméra
