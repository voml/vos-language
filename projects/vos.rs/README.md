# VOS for Rust

The Rust workspace is the reference implementation of VOS: parsing, source normalization, typed ASTs, diagnostics,
catalog identity, optional inspection, and code generation behind one public facade.

Applications and hosts should depend on **`vos`**. The other crates are implementation boundaries and are intentionally
not published as competing entry points.

## Quick start

Parse a schema document through `vos::parser`:

```rust
let source = r#"
table User {
    @@user_id: uuid,
    @user_name: utf8,
}
"#;

let document = vos::parser::parse_document(source) ?;
println!("parsed {} top-level items", document.items.len());
# Ok::<(), vos::ast::Diagnostics>(())
```

Operation programs have a dedicated entry point:

```rust
let program = vos::parse_program(
"let users = User.filter(user => user.enabled == true).collect()",
) ?;
```

When reporting errors, keep the source attached so spans remain meaningful:

```rust
match vos::parser::parse_document(source) {
Ok(document) => { /* use the typed document */ }
Err(diagnostics) => {
for error in vos::report_diagnostics("schema.vos", source, diagnostics) {
eprintln ! ("{error:?}");
}
}
}
```

## What the facade exposes

| Path                         | Purpose                                                          |
|------------------------------|------------------------------------------------------------------|
| `vos::parser`                | Schema and program parsing, checks, and source-aware diagnostics |
| `vos::ast`                   | Typed documents, expressions, operations, spans, and diagnostics |
| `vos::inspect`               | Optional policy checks layered after baseline validity           |
| `vos::generator`             | Dejavu-backed artifact generation                                |
| `vos::catalog_from_document` | Initial stable field-identity catalog IR                         |
| `vos::normalize_source`      | Source normalization used by conformance                         |

Baseline parsing decides whether a program is valid. Inspection is a separate, configurable layer for policies such as
preferring explicit association loads; it must not redefine valid VOS semantics.

## Workspace architecture

| Crate           | Role                                                           | Public dependency? |
|-----------------|----------------------------------------------------------------|--------------------|
| `vos`           | Stable facade and re-exports                                   | **Yes**            |
| `vos-ast`       | Schema, expression, operation, diagnostic, and catalog types   | No                 |
| `vos-parser`    | Normalization, parser, semantic checks, and miette integration | No                 |
| `vos-inspect`   | Configurable post-check inspection rules                       | No                 |
| `vos-generator` | AOT-preferred Dejavu generation helpers                        | No                 |

This boundary lets the implementation evolve without forcing hosts to track crate-level refactors.

## Conformance

The parser is tested against shared fixtures, including normalized source, AST JSON, diagnostics, and catalog snapshots.

```bash
cargo test -p vos-parser --test conformance
```

Read the [fixture contract](../../specifications/fixtures/README.md) before intentionally updating goldens.

## Development

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

New tests belong under each crate's `tests/` directory. Language changes should include focused tests plus a fixture
when the behavior is part of the cross-host contract.

Return to the [project overview](../../readme.md) for the language model, TypeScript tools, and editor integrations.
