# UUID v7 (mandatory)

VOS / Iris expose a single id generator: **`uuid()`** in operation programs and the
equivalent host API on Rust (`vos::uuid()`, `iris::uuid()`).

## Contract

| Rule | Detail |
|------|--------|
| Version | **UUID v7 only** (RFC 9562). No v1/v4/v6 helpers in the language or ORM. |
| Wire shape | Canonical hyphenated string in application JSON; Iris MySQL adapter stores schema `uuid` as `BINARY(16)`. |
| Primary keys | Any `@@field: uuid` PK must be assigned with `uuid()` (or a fixed v7 constant in seed), never random v4. |

## Why v7 (page splits)

Random UUID **v4** primary keys are uniformly distributed. On InnoDB (and any B-tree
ordered by PK), each insert lands on a **random leaf page**. The engine must:

- split pages when a leaf fills,
- rebalance parent nodes,
- evict hot pages from the buffer pool,

so sustained insert rates **collapse** compared with monotonic keys (auto-increment,
**UUID v7**, snowflake, etc.).

UUID **v7** embeds a millisecond timestamp in the high bits, so new ids are mostly
**append-only** at the right edge of the clustered index — same locality as
auto-increment, without exposing sequential integers.

This is why Iris treats `uuid()` as v7-only: using v4 for PK columns is an operational
bug, not a stylistic choice.

## Fixtures

Operation fixtures that call `uuid()` (`insert_construct`, etc.) assume v7 semantics.
Hosts evaluating `uuid()` at runtime must match `vos::uuid()`.
