# Diagnostic Fixtures

Small, intentional failures that keep VOS errors stable, precise, and useful.

Diagnostics are a public language surface. A good failure should identify the right source bytes, explain the violated
rule in VOS terms, and offer a practical hint when one exists. These fixtures make those qualities reviewable.

The default parser is `parse_document`. Program failures include a `*.kind.json` sidecar selecting `parse_program`.

## Current cases

| Stem                      | Protected behavior                                          |
|---------------------------|-------------------------------------------------------------|
| `missing_table_id`        | A persistent table requires a primary key                   |
| `uuid_wrong_case`         | Built-in type names are case-sensitive (`uuid`, not `Uuid`) |
| `unknown_reference`       | A reference target must exist                               |
| `duplicate_primary`       | A table cannot declare two primary fields                   |
| `duplicate_field`         | Field names are unique within a declaration                 |
| `duplicate_type`          | Top-level type names cannot collide                         |
| `star_outside_projection` | Bare `*` is valid only in projection context                |
| `macro_outside_ddl`       | Durable `macro` declarations require a DDL session          |

## Adding a diagnostic

Prefer the smallest source that triggers exactly the intended rule. Assert a stable diagnostic code when the condition
is part of the language contract, keep spans tight, and write hints as actions rather than restatements of the error.

Avoid cascading noise: if one malformed token causes several secondary errors, improve recovery or choose a fixture that
clearly records why the cascade is expected.

See the parent [conformance guide](../README.md) for the JSON shape, span rules, and golden update workflow.
