# `vos`

The stable Rust facade for Virtual Object Schema.

Use this crate when building a database host, compiler, generator, CLI, or developer tool that needs to understand
`.vos` source. Oak owns lexing and parsing. This facade exposes Oak-backed contract input together with the legacy
compatibility surface while downstream semantic migration is in progress.

## Oak-backed contract input

New integrations should begin with `vos::parse_oak`, which invokes Oak and returns
`vos::contract::ContractInput`. VOS semantic consumers must operate on that Oak output and must not add another
parser or reparse the source.

## Parse a schema

```rust
let source = r#"
table Article {
    @@article_id: uuid,
    title: utf8,
    author: &User,
}
"#;

let document = vos::parser::parse_document(source) ?;
# Ok::<(), vos::ast::Diagnostics>(())
```

For expression and operation programs, use `vos::parse_program`:

```rust
let program = vos::parse_program(
"let articles = Article.filter(x => x.published == true).collect()",
) ?;
```

## Public modules

- `vos::parse_oak` is the new Oak-owned parser entry for contract consumers.
- `vos::parser` is a legacy compatibility surface and must not gain new syntax features.
- `vos::ast` contains typed schema, expression, operation, catalog, span, and diagnostic structures.
- `vos::inspect` runs optional policy checks after baseline parsing succeeds.
- `vos::generator` renders artifacts through Dejavu templates.

Convenience exports include `normalize_source`, `parse_program`, `catalog_from_document`, and miette-compatible
diagnostic reporters.
- `vos::uuid()` generates **UUID v7 only** (VOS builtin `uuid()`). Do not use v4 for PK columns — see
  [`specifications/uuid-v7.md`](../../../specifications/uuid-v7.md) (page-split rationale).

## Integration rule

Applications should depend on this facade rather than `vos-ast`, `vos-parser`, `vos-inspect`, or `vos-generator`
directly. That keeps host code on the supported surface while internal crate boundaries evolve.

See the [Rust workspace guide](../README.md) for architecture, conformance, and development commands, or the
root [VOS overview](../../../readme.md) for the language story.
