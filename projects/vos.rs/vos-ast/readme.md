# `vos-ast`

The internal typed representation of VOS schemas and operation programs.

This crate defines the shared data structures that connect parsing, inspection, generation, conformance, and database
hosts: documents, declarations, expressions, query stages, diagnostics, source spans, and stable catalog identity.

## What lives here

- schema items such as tables, classes, enums, flags, and obsolete declarations;
- type expressions for references, optionals, lists, and built-in values;
- expression and operation IR, including projections, query stages, patches, and execution boundaries;
- `Diagnostic`, `Diagnostics`, and byte-oriented `Span` values;
- catalog types such as `TypeId`, `FieldId`, `VirtualFieldIndex`, and revision counters;
- stable diagnostic code constants shared by parsers and hosts.

The AST is serializable because conformance fixtures compare exact output across changes. Source text itself is kept in
normalized `.vos` goldens rather than duplicated in serialized documents.

## Dependency boundary

`vos-ast` is an implementation crate and is not published independently. Application code should use the public [
`vos`](../vos) facade and reach these types through `vos::ast`.

When changing a public structure, review parser construction, serde output, catalog generation, and conformance fixtures
together. A seemingly local field change can alter the cross-host language contract.

See the [Rust workspace README](../README.md) and [fixture contract](../../../specifications/fixtures/README.md) for the
surrounding architecture.
