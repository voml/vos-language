# `@game-gpt/vos-skills`

Agent Skills that help **you** write and review **VOS** (Virtual Object Schema) `.vos` files — whether your application
is TypeScript, Rust, or another language that consumes generated bindings.

## Who this is for

You model data and APIs in `.vos` text. Your AI assistant uses this skill to draft tables, DTOs, enums, and services
correctly. **Installing the skill does not depend on your app's programming language** — only `npx` needs Node.js.

## Install the skill

```bash
# Project-local (recommended)
npx skills add @game-gpt/vos-skills --skill vos-language -y

# Preview contents
npx skills add @game-gpt/vos-skills --list

# All projects on this machine
npx skills add @game-gpt/vos-skills --skill vos-language -y -g
```

Requires [Node.js](https://nodejs.org/) 18+ for the installer only.

Compatible with Cursor, Claude Code, VS Code (Agent mode), and other [Agent Skills](https://agentskills.io/) hosts.

## Use VOS in your project (by language)

The skill includes a full guide at `skills/vos-language/references/hosts.md`. Summary:

| You build with…             | Validate / integrate                                            |
|-----------------------------|-----------------------------------------------------------------|
| **TypeScript / JavaScript** | `npm install @game-gpt/vos` → `checkSource()`                   |
| **Rust**                    | `vos = "0.1"` in Cargo.toml → `vos::parser::parse_document`     |
| **Iris ORM**                | `@yydb/iris` + `iris check` / `iris generate` on `.iris` schema |
| **YYDB / YYDS**             | Host `check` / CLI (native `.vos`)                              |
| **Other languages**         | Edit `.vos` in repo; validate via CI or product codegen         |
| **Any editor**              | VOS syntax extension in VS Code / Cursor                        |

After the skill is installed, ask your agent:

- "Add a `table` for invoices linked to `Customer`"
- "How do I check this `.vos` file in my Rust project?"
- "Review this service definition for VOS mistakes"

## What's in the package

| Skill          | Purpose                                                   |
|----------------|-----------------------------------------------------------|
| `vos-language` | User-facing help for `.vos` schema, classes, and services |

## Maintainers

Source: [voml/vos-language](https://github.com/voml/vos-language) → `projects/vos.ts/vos-skills`

```bash
pnpm --filter @game-gpt/vos-skills typecheck
npx skills add ./projects/vos.ts/vos-skills --list
```
