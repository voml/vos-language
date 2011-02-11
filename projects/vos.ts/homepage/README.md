# VOS Homepage and Playground

The VOS homepage turns the language contract into an approachable product experience: a concise language tour, visual
explanations of relationships and lifecycle, and an interactive playground with consistent syntax highlighting.

This is a private workspace application, not a published library. It consumes `@game-gpt/vos` for browser-side feedback
and `@game-gpt/vos-textmate` for Shiki highlighting.

## Run locally

From the repository root:

```bash
pnpm install
pnpm --filter @game-gpt/vos-homepage dev
```

Create and preview a production build with:

```bash
pnpm --filter @game-gpt/vos-homepage build
pnpm --filter @game-gpt/vos-homepage preview
```

## Content principles

- Lead with the problem VOS solves before listing syntax.
- Keep examples valid against the current language surface.
- Explain VOS as a native schema and operation language, never as a SQL wrapper.
- Keep English and Chinese locale keys aligned when changing visible content.
- Reuse the shared TextMate grammar; do not add homepage-only tokenization rules.
- State TypeScript checker limitations honestly and point advanced parsing needs to the Rust facade.

## Structure

| Area                           | Purpose                                |
|--------------------------------|----------------------------------------|
| `src/views/HomePage.vue`       | Main language narrative                |
| `src/views/ModelPage.vue`      | Deeper model exploration               |
| `src/views/PlaygroundPage.vue` | Interactive source checking            |
| `src/samples.ts`               | Curated VOS examples                   |
| `src/i18n`                     | English and Chinese product copy       |
| `src/components`               | Site, home, and reusable UI components |

See the [TypeScript workspace README](../README.md) for package relationships and the
root [VOS README](../../../readme.md) for the project-wide story.
