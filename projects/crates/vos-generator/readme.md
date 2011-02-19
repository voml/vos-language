# `vos-generator`

Dejavu-backed artifact generation for VOS hosts.

The generator turns typed VOS context into repeatable output such as Rust modules. Templates are preferred ahead of
handwritten string assembly because generated artifacts need the same reviewability and determinism as the language
structures that feed them.

## Generation modes

- **`aot`** is the default: templates are prepared ahead of time so runtime work focuses on rendering.
- **`dyn`** parses template source at runtime for debugging and comparison workflows.

The crate exposes general render helpers plus Rust-oriented helpers such as file headers, structs, field context
conversion, and JSON template contexts.

## Design boundary

Generation consumes established VOS semantics; it does not own a private schema dialect or attach hidden meaning that
parsers and other hosts cannot observe. Generated code is one possible artifact, not the definition of the language.

Applications should access this crate through `vos::generator` from the public [`vos`](../vos) facade. The crate is
internal and is not published independently.

See the [Rust workspace README](../README.md) for development and verification commands.
