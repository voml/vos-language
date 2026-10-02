# VOS authoring reference

Grammar and evolution quick reference. For **installing the skill** and **using VOS in TypeScript, Rust, Iris, or other
stacks**, read [hosts.md](hosts.md) first.

## What goes where

| Construct                   | User intent                                       |
|-----------------------------|---------------------------------------------------|
| `table`                     | Data the application stores long-term             |
| `class`                     | Request, response, filter, or view payload        |
| `service`                   | Callable API surface (RPC / HTTP-style contracts) |
| `enums` / `flags` / `union` | Closed choices, bit sets, tagged alternatives     |

A `service` may return a shared `table` or domain type when that entity is intentionally public on the API.

## Namespaces and imports

Group files by domain. Import before referencing types defined elsewhere:

```vos
namespace app::organization;
using app::identity::User;

table OrganizationMember {
    @@member_id: uuid,
    user_id: &User,
}
```

Prefer `schemas/<domain>/*.vos` or the layout the user's project already uses — do not impose a repo-specific tree
unless they ask.

## Compatibility (for the user's team)

**Usually backward compatible**

- Add optional fields
- Add new enum values with new numbers
- Add new tables or new nullable columns

**Coordinate before shipping**

- Remove or rename fields or tables
- Change types or nullability
- Renumber enum values
- Change existing service signatures

## Evolving a live schema

When the user's stored model changes:

- **Add** tables and nullable fields freely when their migration tool allows it.
- **Mark deprecation** with `obsolete table Name;` or `obsolete field Table.field;` before removal where the toolchain
  supports it.
- **Non-null new columns** may need a backfill strategy — the user's migration product defines how; do not invent hidden
  defaults in schema without asking.
- **Never put secrets** (API keys, connection strings) in `.vos` files.

If the user syncs `.vos` to an external database product, their local `.vos` remains authoritative — do not suggest
reverse-engineering schema from the remote catalog back into VOS without review.

## Common mistakes to catch

- `Uuid` instead of `uuid`
- `enum` instead of `enums`, or `type` instead of `class`
- `T[]` instead of `[T]`
- Missing primary key on a `table`
- Reference `&User` without a resolvable `User` table in scope
- Positional service parameters

## Learn more

- Language and ecosystem: [github.com/voml/vos-language](https://github.com/voml/vos-language)
- Values and documents: [github.com/voml/von-language](https://github.com/voml/von-language)
