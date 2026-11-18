# Schema and DDL Fixtures

Golden examples for VOS documents parsed with `parse_document`.

These cases protect the declarative heart of the language: namespaces, stored tables, non-persistent classes, keys,
references, closed types, explicit obsolescence, and the catalog identity derived from a valid document.

## Current cases

| Stem              | What it demonstrates                                            |
|-------------------|-----------------------------------------------------------------|
| `basic_table`     | Namespace, primary and unique shortcuts, and an optional field  |
| `bracket_primary` | The long-form `[primary]` field attribute                       |
| `types_bundle`    | Enums, flags, classes, tables, and obsolete field/table symbols |

Successful schema cases normally include `*.ast.json`, empty diagnostics, and `*.catalog.json`. The AST records the
source-facing structure; the catalog records durable `FieldId` and virtual-slot identity used beyond source spelling and
order.

## Adding coverage

Keep each source file centered on one schema capability. Add a separate fixture when a new feature has materially
different parsing, diagnostics, or catalog behavior. After generating companions, review field order, spans, type
wrappers, IDs, virtual indexes, and revisions instead of checking only that the test passes.

See the parent [conformance guide](../README.md) for file conventions and update commands.
