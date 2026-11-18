# `vos-format`

VOS formatter. Consumers depend on this crate directly; schema semantics and the resolved contract come from the
Boost `schema-input-vos` crate.

Oak owns the VOS lexer, parser, CST, spans and recovery. This crate owns no lexer, parser or AST of its own: it consumes
`oak-vos` output.

## Modes

| Mode | Behaviour |
| --- | --- |
| `Mode::Preserve` | Re-emits the exact source bytes from the lossless Oak CST, so it is also a round-trip check of the frontend. |
| `Mode::Canonical` | Rewrites layout for `namespace`, `using`, `table` and `class`, and normalizes static paths to `::`. |

```rust
let canonical = vos_format::format(source, vos_format::Mode::Canonical)?;
let preserved = vos_format::format(source, vos_format::Mode::Preserve)?;
assert_eq!(preserved, source);
```

Canonical mode keeps the author's attribute spelling, including custom arguments such as `[backfill("legacy, value")]`.
Type aliases such as `vec<3>` stay as written: rewriting them to the builtin spelling is a VOS semantic decision, not a
formatting one.

Declarations Oak does not decompose yet (`enums`, `flags`, `const`, `obsolete`, `service`, `micro`, `macro`, `query`,
`udf`) are passed through verbatim, so canonical mode never loses or rewrites source it does not understand.

See the repository readme for architecture and development commands.
