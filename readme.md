# VOS - Virtual Object Schema

VOS is a versioned schema language for strongly typed shapes. It is the same
language historically presented as Atlas **Iris** (persistence) and **Hermes**
(communication): one grammar, two facets.

It pairs with [VON](../von-language) (**Virtual Object Notation**) for values.
Brand, product boundary, runtime, and migration history stay independent of any
single database product; shared schema semantics align through versioned
contracts and conformance tests.

### Language surface (from Atlas schema)

| Item | Role |
| --- | --- |
| `namespace` / `using` | Isolation and cross-file type imports |
| `table` | Persistence facet (Iris) |
| `class` | Shared DTO / domain types |
| `enums` / `flags` / `union` | Numeric enums, bitflags, tagged unions |
| `service` | Communication facet (Hermes); named parameters required |
| `const` / `obsolete` | Shared constants and explicit removals |

Field layout: `[attr] name: Type = default`, with `&T` = PK reference, `T` =
inline value, `T?` optional, `[T]` list. Shortcuts `@@id` / `@unique` expand to
`[primary]` / `[unique]`.

## Repository layout

```text
vos-language/
├── .github/workflows/     CI for Rust (and future TypeScript)
├── projects/
│   ├── vos.rs/            Rust crates
│   └── vos.ts/            TypeScript packages (placeholder)
└── README.md
```

### Rust (`projects/vos.rs`)

| Crate | Role |
| --- | --- |
| `vos` | Public facade |
| `vos-ast` | Schema syntax data types |
| `vos-parser` | Lexer / parser boundary |

The `vos` facade currently re-exports the upstream Iris implementation while
the remote repository migrates to the VOS name.

### TypeScript (`projects/vos.ts`)

| Package | Role |
| --- | --- |
| `@game-gpt/vos` | Schema toolkit (minimal source checks for now) |
| `@game-gpt/vos-homepage` | Language site + playground |

## Quick start

### Homepage

```bash
cd projects/vos.ts
pnpm install
pnpm dev
```

Open http://localhost:5175/

### Rust

```bash
cd projects/vos.rs
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## CI

| Workflow | Path filter | Checks |
| --- | --- | --- |
| `vos-rs.yml` | `projects/vos.rs/**` | fmt / check / clippy / test |

## Relation to VON

| Language | Full name | Role |
| --- | --- | --- |
| VON | Virtual Object Notation | Values / documents |
| VOS | Virtual Object Schema | Types / constraints |

## License

MPL-2.0 (see [`License.md`](./License.md)).
