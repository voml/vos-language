# `vos`

The stable Rust facade for Virtual Object Schema.

Use this crate when building a database host, compiler, generator, CLI, or developer tool that needs to understand
`.vos` source. It presents the parser, AST, inspection engine, generator, diagnostics, and catalog helpers through one
dependency boundary.

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

- `vos::parser` parses documents and programs and produces source-aware errors.
- `vos::ast` contains typed schema, expression, operation, catalog, span, and diagnostic structures.
- `vos::inspect` runs optional policy checks after baseline parsing succeeds.
- `vos::generator` renders artifacts through Dejavu templates.

Convenience exports include `normalize_source`, `parse_program`, `catalog_from_document`, and miette-compatible
diagnostic reporters.

## Integration rule

Applications should depend on this facade rather than `vos-ast`, `vos-parser`, `vos-inspect`, or `vos-generator`
directly. That keeps host code on the supported surface while internal crate boundaries evolve.

See the [Rust workspace guide](../README.md) for architecture, conformance, and development commands, or the
root [VOS overview](../../../readme.md) for the language story.
