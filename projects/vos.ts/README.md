# VOS for TypeScript

This workspace brings VOS into TypeScript projects, browser tooling, documentation, and editors. It contains the public
TypeScript API, the VOS homepage and playground, the shared TextMate grammar, and the VS Code/Cursor extension that
consumes that grammar.

## Packages

| Package                                  | Purpose                                                        | Status                     |
|------------------------------------------|----------------------------------------------------------------|----------------------------|
| [`@game-gpt/vos`](./vos)                 | Public TypeScript source-checking API                          | Lightweight checker        |
| [`@game-gpt/vos-homepage`](./homepage)   | Homepage, language tour, and browser playground                | Private application        |
| [`@game-gpt/vos-textmate`](vos-textmate) | Shared `.vos` grammar for Shiki and editor tooling             | Internal workspace package |
| [`@game-gpt/vos-skills`](./vos-skills)   | Agent Skills for **users** authoring `.vos` (`npx skills add`) | Public npm package         |
| [`vscode-vos`](./vscode-vos)             | VS Code/Cursor syntax highlighting                             | Private extension package  |

The Rust implementation is currently the reference parser. The TypeScript API deliberately describes its narrower
supported surface instead of pretending to offer full AST or conformance parity.

## Try the public API

```ts
import {checkSource} from "@game-gpt/vos";

const result = checkSource(`
table User {
    @@user_id: uuid,
    @user_name: utf8,
}
`);

if (!result.ok) {
    for (const diagnostic of result.diagnostics) {
        console.error(`${diagnostic.line}: ${diagnostic.message}`);
    }
}
```

Today `checkSource` performs fast structural checks suitable for a browser playground or early editor feedback. Use the
Rust facade for the canonical typed AST, full semantic diagnostics, catalog IR, or conformance-grade parsing.

## Run the homepage

From the repository root:

```bash
pnpm install
pnpm --filter @game-gpt/vos-homepage dev
```

The homepage uses the same TextMate grammar as the editor extension, so examples stay visually consistent in Shiki, VS
Code, and Cursor.

## Work on highlighting

The source grammar lives in [`vos-textmate/vos.tmLanguage.json`](vos-textmate/vos.tmLanguage.json). After changing it,
sync the extension copy:

```bash
pnpm --filter @game-gpt/vos-textmate sync-vscode
```

Do not hand-edit a second grammar in the homepage or extension.

## Verify the workspace

```bash
pnpm -r typecheck
pnpm -r test
pnpm -r build
```

Some packages may not define every script; pnpm runs the scripts that exist. For language-level compatibility, refer to
the shared [conformance fixtures](../../specifications/fixtures/README.md).

See the repository [README](../../readme.md) for the language story, Rust implementation, and contribution expectations.
