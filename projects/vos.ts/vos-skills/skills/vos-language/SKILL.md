---
name: vos-language
description: >-
  Help the user write, review, or fix VOS (.vos) schema files for their application
  — tables, keys, references, DTO classes, enums, and service contracts. Use when
  the user mentions .vos files, Virtual Object Schema, data models, API shapes,
  or wants to define persisted entities and request/response types in one place.
license: MPL-2.0
compatibility: >-
  Install skill with Node.js 18+ (`npx skills add`). VOS source applies to any
  application language; see references/hosts.md for TypeScript, Rust, Iris, and editor setup.
metadata:
  author: game-gpt
  version: "0.1.0"
---

# vos-language

Help the **user** express their application's data model and service contracts in **VOS** (Virtual Object Schema). They
edit plain `.vos` text files; your job is to draft or fix that source correctly — not to maintain the VOS compiler or
repository tooling.

## What the user is doing

VOS lets them describe, in one readable grammar:

- **what is stored** (`table`, keys, references),
- **what travels on the wire** (`class`, `service`),
- **how values are shaped** (`enums`, `flags`, `union`).

Values at runtime usually live in [VON](https://github.com/voml/von-language); VOS defines the types and contracts.

## Install and stack-specific usage

**First time:** if the user has not installed this skill, show them [references/hosts.md](references/hosts.md) — it
covers `npx skills add` (works in **any** project) and how to validate `.vos` in **TypeScript, Rust, Iris, YY hosts, and
other languages**.

If you do not know their language or product stack, ask once, then follow the matching section in `hosts.md`. Do not
assume they use Rust or a specific CLI.

## How to help

1. **Clarify the domain** — suggest a stable `namespace` (for example `billing::invoices`).
2. **Model storage first** — tables, primary/unique keys, then `&T` references between tables.
3. **Add wire types next** — request/response `class` types and `service` methods.
4. **Keep one source of truth** — change `.vos` files; tell the user to re-run **their project's** generate/check
   command if they use one. Do not edit generated Rust, TypeScript, OpenAPI, or ORM output by hand.
5. **Prefer small, reviewable diffs** — one domain or one feature per change when possible.

If the user's toolchain reports an error, use the message and line it gives; do not invent SQL or alternate schema
dialects to work around VOS.

## Persistence (`table`)

```vos
table User {
    @@user_id: uuid,
    @user_name: utf8,
    display_name: utf8,
    manager: &User? = null,
}
```

| Form                           | Meaning                                            |
|--------------------------------|----------------------------------------------------|
| `@@field` or `[primary] field` | Primary key                                        |
| `@field`                       | Unique key                                         |
| `&T` / `&T?`                   | Reference to another table's primary key           |
| bare `T`                       | Inline full value, not a foreign key               |
| `uuid`                         | Lowercase builtin only (`Uuid` is invalid)         |
| `DateTime<UTC>`                | Time point; avoid raw `i64` timestamps             |
| `enums` / `flags`              | Explicit numeric values; `flags` use powers of two |

Passwords, pagination filters, and one-off command options belong in dedicated `class` types — not as extra columns on a
`table` unless they are truly stored fields.

## Communication (`class`, `service`)

```vos
namespace app::organization;
using app::identity::User;

class CreateMemberRequest {
    organization_id: uuid,
    user_id: uuid,
}

service OrganizationService {
    create_member(request: CreateMemberRequest) -> OrganizationMember
}
```

- Use `class`, not `type`. Use `enums`, not `enum`.
- Lists are `[T]`, not `T[]`.
- Service methods use **named parameters**: `create(request: T) -> R`. Positional `create(T)` is invalid.
- `stream<T>` streams values over time; `[T]` is a one-shot list.
- JSON-RPC is the default service style unless the user's project documents REST attributes such as
  `[post("/path")]`.

## Safe changes

| Usually safe                       | Needs explicit review              |
|------------------------------------|------------------------------------|
| New optional field                 | Remove or rename field             |
| New enum variant with a new number | Renumber or delete enum variant    |
| New table or new nullable column   | Tighten nullability or change type |
| New service method                 | Change existing method signature   |

When removing persisted fields or tables, use `obsolete field` / `obsolete table` symbols instead of silently deleting
definitions. See [references/reference.md](references/reference.md) for grammar notes and
[references/hosts.md](references/hosts.md) for per-language install and check commands.

## Do not

- Write SQL DDL/DML or propose SQL as a substitute for `.vos` in VOS-native workflows.
- Hand-edit generated client, server, or OpenAPI artifacts — always fix the `.vos` source.
- Mix persistence and wire concerns (for example storing a password hash on a login **request** `class`).
- Guess toolchain-specific commands — ask which check/generate command the user's stack provides if unknown.
