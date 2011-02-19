# Install and use VOS (by stack)

This guide is for **people building applications** — not for contributors to the `vos-language` repository.

## Step 1 — Install the Agent Skill (any language)

The skill teaches your **AI assistant** how to write `.vos` files. It does not replace your app's VOS library.

**Requires:** [Node.js](https://nodejs.org/) 18+ (only for the one-time `npx` installer).

```bash
# Install into the current project (recommended)
npx skills add @game-gpt/vos-skills --skill vos-language -y

# Preview skills in the package
npx skills add @game-gpt/vos-skills --list

# Install for all projects on this machine
npx skills add @game-gpt/vos-skills --skill vos-language -y -g
```

Works with agents supported by the [Agent Skills](https://agentskills.io/) ecosystem (for example Cursor, Claude Code,
VS Code with Copilot Agent mode, and others that honor `npx skills`).

**After install**, ask in chat:

- "Create a `table` for orders with a reference to `Customer`"
- "Review this `.vos` file for key and naming mistakes"
- "Add a `service` method to list active users"

The skill file lands under your project's agent skills directory (for example `.agents/skills/vos-language/`).

---

## Step 2 — Use VOS in your project

Pick the row that matches **your application language**. The `.vos` **source is the same**; only the check/generate
tooling differs.

| Your stack                  | Install / validate                                        | Typical workflow                                                                |
|-----------------------------|-----------------------------------------------------------|---------------------------------------------------------------------------------|
| **Any editor**              | [VS Code / Cursor extension](#editor-syntax-highlighting) | Edit `.vos` → run your stack's check → use generated code                       |
| **TypeScript / JavaScript** | [`@game-gpt/vos`](#typescript--javascript)                | `checkSource()` for quick feedback; use your app's generator for full semantics |
| **Rust**                    | [`vos` crate](#rust)                                      | `parse_document` / product CLI for full diagnostics                             |
| **Iris ORM**                | [`@yydb/iris`](#iris-orm-typescript--rust-hosts)          | `.iris` schema (VOS syntax) → `iris check` / `iris generate`                    |
| **YYDB / YYDS native**      | Product docs                                              | `.vos` is native DDL/operations — use host `check` / CLI                        |
| **Other languages**         | [No local parser yet](#other-languages-c-kotlin-go-…)     | Edit `.vos` in repo; validate in CI or via Rust/product host                    |

---

### Editor (syntax highlighting)

Install the **VOS** extension in VS Code or Cursor for `.vos` highlighting and bracket matching.

- Marketplace: search for **VOS** (publisher `game-gpt`) when published
- From source: build `vscode-vos` in [voml/vos-language](https://github.com/voml/vos-language)

Highlighting helps you read schema; it does not replace semantic checking.

---

### TypeScript / JavaScript

Add the lightweight checker to your app or tooling repo:

```bash
npm install @game-gpt/vos
# or
pnpm add @game-gpt/vos
```

```ts
import {checkSource} from "@game-gpt/vos";

const result = checkSource(`
table User {
    @@user_id: uuid,
    @user_name: utf8,
}
`);

if (!result.ok) {
    for (const d of result.diagnostics) {
        console.error(`line ${d.line}: ${d.message}`);
    }
}
```

`checkSource` catches structural issues quickly (braces, top-level items, obvious service shape errors). For full AST,
catalog identity, and conformance-grade errors, use the **Rust** `vos` crate or your product's check command.

---

### Rust

Add the reference parser to `Cargo.toml`:

```toml
[dependencies]
vos = "0.1"
```

```rust
let document = vos::parser::parse_document(source) ?;
// operation programs:
let program = vos::parse_program(source) ?;
```

Use this when you embed VOS in a Rust service, CLI, or custom toolchain. If you use **Iris** or **YYDB**, prefer their
facade commands instead of wiring `vos` directly unless you are building a host.

---

### Iris ORM (TypeScript / Rust hosts)

[Iris](https://github.com/yy-database/iris-orm) consumes **VOS semantics**; schema files often use the `.iris`
extension.

```bash
npm install @yydb/iris
# Agent help for Iris workflows (optional, separate package):
npx skills add @yydb/iris-skills
```

Typical commands (from your app root):

```bash
iris check path/to/schema.iris
iris generate path/to/schema.iris
```

Edit `.iris` / `.vos` source — not generated `generated/` client code. For Iris-specific migrations and topology, use
`@yydb/iris-skills`; **`vos-language`** skill covers the shared VOS grammar either way.

---

### YYDB / YYDS native hosts

On native YY stacks, `.vos` is the **DDL and operation language**. Use the host's documented `check`, `inspect`, and
execute paths — do not introduce SQL as a parallel schema truth.

Ask the user which YY product and CLI version they run if commands differ.

---

### Other languages (C#, Kotlin, Go, …)

There is no separate first-party VOS parser package in every language today. You can still:

1. Author `.vos` files in your repo (same grammar everywhere).
2. Run validation in CI via Rust `vos`, a product host, or codegen that fails on invalid input.
3. Consume **generated** bindings produced by your stack's Dejavu / Iris / custom generator.

Tell the agent which generator your project uses so it does not invent a second schema format.

---

## Step 3 — Everyday workflow (all stacks)

```text
Install skill (once)
    ↓
Draft or edit .vos / .iris with AI + editor
    ↓
Run your stack's check command
    ↓
Run generate / migrate if applicable
    ↓
Use generated types/clients in application code (TS, Rust, …)
```

**Rules that never change:**

- `.vos` (or `.iris`) is the source of truth — not SQL, not hand-edited generated files.
- One namespace per domain; import with `using` before referencing remote types.
- Breaking schema changes need team review; mark deprecations with `obsolete` before removal.

---

## Related packages

| Package                | Who it is for                                         |
|------------------------|-------------------------------------------------------|
| `@game-gpt/vos-skills` | **This package** — AI help writing VOS (any language) |
| `@game-gpt/vos`        | TS/JS lightweight source check                        |
| `vos` (Rust)           | Full parser / AST / inspect                           |
| `@yydb/iris-skills`    | Iris ORM agent workflows (optional add-on)            |
| `@yydb/iris`           | Iris typed data access                                |

More: [github.com/voml/vos-language](https://github.com/voml/vos-language)
