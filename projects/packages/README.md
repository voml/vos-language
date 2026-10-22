# VOS for TypeScript

This workspace brings VOS into TypeScript projects, browser tooling, documentation, and editors. It contains the public
TypeScript API, the VOS homepage and playground, the shared TextMate grammar, and the VS Code/Cursor extension that
consumes that grammar.

## Packages

| Package                                        | Purpose                                                                 | Status                     |
|------------------------------------------------|-------------------------------------------------------------------------|----------------------------|
| [`@voml/vos`](./vos)                           | Public TypeScript declaration and type-modelling API (`v.*`, `Infer`)    | Modelling surface          |
| [`@voml/vos-language`](./vos-language)         | Language API (`.vos` source checks and diagnostics) backed by the Rust wasm core | Wasm-backed checker        |
| [`@game-gpt/vos-homepage`](./homepage)         | Homepage, language tour, and browser playground                          | Private application        |
| [`@game-gpt/vos-textmate`](vos-textmate)       | Shared `.vos` grammar for Shiki and editor tooling                       | Internal workspace package |
| [`@game-gpt/vos-skills`](./vos-skills)         | Agent Skills for **users** authoring `.vos` (`npx skills add`)           | Public npm package         |
| [`vscode-vos`](./vscode-vos)                   | VS Code/Cursor syntax highlighting                                       | Private extension package  |

The Rust implementation is currently the reference parser. The TypeScript API deliberately describes its narrower
supported surface instead of pretending to offer full AST or conformance parity.

## Model a table

```ts
import { v, type Infer } from "@voml/vos";

const User = v.table("User", {
    user_id: v.uuid().primary(),
    user_name: v.utf8().unique(),
});

type UserRow = Infer<typeof User>;
```

## Check source

Source checks belong to the language package, not to the modelling package:

```ts
import { checkSource } from "@voml/vos-language";

const result = await checkSource(`
table User {
    @@id: uuid,
    name: utf8,
}
`);

if (!result.ok) {
    for (const diagnostic of result.diagnostics) {
        console.error(`${diagnostic.line ?? "L?"}: ${diagnostic.message}`);
    }
}
```

The check runs in the Rust WebAssembly core staged into `lib/` by `scripts/build-wasm.mjs`; the TypeScript surface only
loads and types it. Use the Rust facade for the canonical typed AST, full semantic diagnostics, catalog IR, or
conformance-grade parsing.

## Run the homepage

**Published:** [vos-language.pages.dev](https://vos-language.pages.dev/) (Cloudflare Pages, `dev` branch → preview at [dev.vos-language.pages.dev](https://dev.vos-language.pages.dev/)).

From the repository root:

```bash
pnpm install
pnpm --filter @game-gpt/vos-homepage dev
```

The homepage is a VMZ application (`@vmz/vmz`, `@vmz/ui`, `@vmz/ui-icons` at npm `0.1.12`; Shiki via
`@vmz/plugin-shiki` + `@game-gpt/vos-textmate/shiki`) with file-based routes for
`/`, `/model`, and `/playground`. Build outputs use **`dist/<target>`** — `dist/cdn` for static CDN upload,
`dist/browser` for local SSR dev, `dist/wechat` for WeChat. From the repo root, `pnpm homepage` writes the static release
to `projects/packages/homepage/dist/cdn`.

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
the shared [conformance fixtures](../../documentation/fixtures/README.md).

See the repository [README](../../readme.md) for the language story, Rust implementation, and contribution expectations.
