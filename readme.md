# VOS

**One language for the shape, identity, and behavior of structured data.**

**Homepage:** [vos-language.pages.dev](https://vos-language.pages.dev/) — language tour, model pages, and browser playground.

## Start with the VOS skill

The fastest way into VOS is through your coding agent. Install the official project-local skill, describe the data or API
you want to model, and let the agent bring the right VOS conventions, validation path, and ecosystem integration into
the conversation.

```bash
npx skills add @game-gpt/vos-skills --skill vos-language -y
```

The installer requires Node.js 18+, but the skill does not depend on your application's language. It works with Agent
Skills hosts such as Cursor, Claude Code, and VS Code Agent mode, whether the project itself uses Rust, TypeScript, Iris,
YYDB, or another VOS consumer.

Once installed, start with the outcome you need rather than assembling a toolchain by hand:

```text
Design a VOS model for customers, invoices, payments, and invoice line items.

Review every .vos file in this project for invalid references, weak field modeling,
and service contracts that expose persistence-only data.

Add VOS validation to this Rust project and explain which checks belong in CI.

Model this API in VOS, then show how its schema should flow into Iris ORM without
duplicating query semantics in generated code.

Explain whether this requirement belongs to VOS, Iris ORM, or Hermes RPC, and draft
the smallest contract that keeps those boundaries intact.
```

Preview the skill before installing it, or install it globally when you intentionally want the same VOS guidance in
every workspace:

```bash
npx skills add @game-gpt/vos-skills --list
npx skills add @game-gpt/vos-skills --skill vos-language -y -g
```

See [`@game-gpt/vos-skills`](./projects/vos.ts/vos-skills) for the package contents and host-specific guidance.

VOS (Virtual Object Schema) is a strongly typed schema and operation language for systems that need more than
disconnected model definitions. A `.vos` file can describe persistent tables, domain objects, references, enums, tagged
unions, service contracts, and the operations that move data through an application.

The result is a source format that stays readable to people while giving parsers, databases, generators, editors, and
conformance tools the same precise contract.

```vos
namespace studio::identity

table User {
    @@user_id: uuid,
    @user_name: utf8,
    display_name: utf8,
    manager: &User? = null,
}

class LoginRequest {
    user_name: utf8,
    password: utf8,
}

service IdentityService {
    login(request: LoginRequest) -> User
}
```

VOS is the schema language in the YY data stack. [VON](https://github.com/voml/von-language) is its companion notation
for values and documents: VOS defines what data means; VON carries the data itself.

## Why VOS exists

Most projects describe the same domain repeatedly: database DDL, application types, validation rules, API payloads,
migration scripts, and generator configuration. Those copies drift because none of them has enough semantic authority to
represent the whole model.

VOS gives that model a single, versionable language:

- **Readable by design.** Field names, types, references, defaults, and constraints stay visible in compact source.
- **Built for durable identity.** Stored fields have stable catalog identity beyond their current spelling or source
  order.
- **Schema and operations belong together.** Typed construction, projections, queries, updates, and DDL share one object
  model.
- **Host-neutral semantics.** Rust, TypeScript, databases, editors, and generators consume one contract instead of
  inventing local dialects.
- **Diagnostics are part of the language.** Stable codes and source spans make errors useful in terminals, editors, CI,
  and products.
- **No SQL-shaped escape hatch.** VOS is a native object and type language, not a cosmetic frontend that lowers into
  SQL.

## A language that grows with the system

VOS covers both the declarations developers recognize immediately and the less visible contracts production systems
eventually need.

| Surface                     | What it models                                                |
|-----------------------------|---------------------------------------------------------------|
| `namespace`, `using`        | Names and cross-file boundaries                               |
| `table`                     | Persistent entities and keys                                  |
| `class`                     | DTOs and non-persistent domain records                        |
| `enums`, `flags`, `union`   | Closed choices, bit flags, and tagged alternatives            |
| `service`                   | Typed RPC and HTTP-facing contracts                           |
| `&T`, `T?`, `[T]`           | References, optional values, and collections                  |
| expressions and operations  | Construction, filtering, projection, mutation, and collection |
| `micro`, `macro`            | Session-local helpers and durable DDL programs                |
| `obsolete`, `Database::...` | Explicit evolution of a live schema                           |

Fields use `[attribute] name: Type = default`. The compact `@@id` and `@name` forms mark primary and unique fields
without hiding their intent.

```vos
let active = User
    .filter(user => user.enabled == true)
    .map(user => user.{
        user_id: user.user_id,
        name: user.display_name,
    })
    .collect()
```

Operations follow the same object and method vocabulary as the schema. References are inferred from use sites; an
explicit `.load(...)` may document or tune intent, but is not required to make a valid relationship meaningful.

## Native execution, not translation

YYDB and YYDS consume VOS as their native DDL and operation language. They do not route VOS through SQL, expose a SQL
compatibility grammar, or require applications to think in relational query strings.

External integrations may project a local VOS model onto another storage technology, but that adapter boundary does not
change the language contract. The invariant is simple: **formal VOS, YYDB, and YYDS surfaces remain SQL-free.**

## Choose your entry point

| You want to...                               | Start here                                               |
|----------------------------------------------|----------------------------------------------------------|
| Let an agent design, review, or integrate VOS | [`@game-gpt/vos-skills`](./projects/vos.ts/vos-skills)  |
| Explore the language in a browser            | [vos-language.pages.dev](https://vos-language.pages.dev/) |
| Parse VOS or integrate it into a Rust host   | [`projects/vos.rs`](./projects/vos.rs)                   |
| Check VOS source from TypeScript             | [`@game-gpt/vos`](./projects/vos.ts/vos)                 |
| Work on the TypeScript packages or homepage  | [`projects/vos.ts`](./projects/vos.ts)                   |
| Add `.vos` highlighting to VS Code or Cursor | [`vos-on-vscode`](./projects/vos.ts/vos-on-vscode)       |
| Use the shared grammar with Shiki            | [`@game-gpt/vos-textmate`](projects/vos.ts/vos-textmate) |
| Validate another implementation              | [`specifications/fixtures`](./specifications/fixtures)   |

The Rust facade is the most complete parser and language implementation today. The TypeScript package currently provides
a lightweight source check and is intentionally narrower; its README states the supported surface explicitly.

## Repository map

```text
vos-language/
├── projects/
│   ├── vos.rs/          Rust facade, parser, AST, inspect, and generation
│   └── vos.ts/          TypeScript API, grammar, extension, and site
├── specifications/
│   └── fixtures/        Shared source, AST, catalog, and diagnostic goldens
├── scripts/             Repository automation
└── projects/vos.ts/vos-skills/   Agent Skills npm package (`@game-gpt/vos-skills`)
```

Public applications should depend on the facade for their host: `vos` in Rust or `@game-gpt/vos` in TypeScript. Internal
Rust crates keep the implementation maintainable; they are not competing public APIs.

## Conformance is a product feature

Language implementations are measured against shared fixtures rather than prose alone. Each case can include normalized
source, serialized AST, diagnostics, and catalog identity. This makes parser behavior reviewable and gives future hosts
a concrete compatibility target.

```bash
cd projects/vos.rs
cargo test -p vos-parser --test conformance
```

See the [fixture guide](./specifications/fixtures/README.md) before adding or updating goldens.

## Contributing

Changes to syntax or semantics should arrive with focused parser coverage and conformance evidence. Keep public APIs
behind the host facade, preserve stable diagnostic behavior, and avoid introducing a second schema dialect in a
generator or adapter.

```bash
cd projects/vos.rs
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

pnpm install
pnpm typecheck
pnpm -r test
```

## License

VOS is available under the [Mozilla Public License 2.0](./License.md).
