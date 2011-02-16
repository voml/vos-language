# Operation Language Fixtures

Golden programs for VOS expressions and object/method operations, parsed with `parse_program`.

VOS operations are not embedded SQL. They use typed construction, value members, static type paths, lazy collection
methods, projections, and explicit execution boundaries that belong to the same language as the schema.

## Current cases

| Stem                 | What it demonstrates                                        |
|----------------------|-------------------------------------------------------------|
| `insert_construct`   | Typed object construction and `.insert()` as distinct steps |
| `filter_map_collect` | A lazy query pipeline ending at an execution boundary       |
| `projection_fields`  | Named projection fields, expressions, and `*` expansion     |
| `update_delete`      | Entity and collection mutation forms                        |
| `static_path_filter` | `Type::method` as the canonical static collection entry     |
| `micro_normalize`    | A session-local `micro` declaration and its call site       |

## Review priorities

Operation ASTs tend to reveal subtle regressions. Check path separators, lambda parameters, projection names, stage
order, expression spans, and the final execution form. A parser that accepts the source but changes any of these may
still have broken a host.

Reference use sites can imply association loading. An explicit `.load(...)` may be inspected as project policy, but
fixtures must not turn it into a universal validity requirement.

Run the dedicated contract with:

```bash
cargo test -p vos-parser --test conformance
```

See the parent [conformance guide](../README.md) for companion files and intentional update instructions.
