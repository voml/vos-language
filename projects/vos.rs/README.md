# vos (Rust)

Rust workspace for **VOS — Virtual Object Schema**.

## Crates

| Crate        | Role                                                                             |
|--------------|----------------------------------------------------------------------------------|
| `vos`        | Public facade (currently re-exports upstream Iris until remote rename completes) |
| `vos-ast`    | Schema syntax data types                                                         |
| `vos-parser` | Lexer / parser boundary                                                          |

## Develop

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

See the repository root [`README.md`](../../README.md) for the language overview.
