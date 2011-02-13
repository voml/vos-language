# VOS Homepage and Playground

The VOS homepage turns the language contract into an approachable product experience: a concise language tour, visual
explanations of relationships and lifecycle, and an interactive playground with structural checks.

This is a private workspace application built with **VMZ** (`@vmz/vmz@0.1.9`), **@vmz/ui**, and **@vmz/ui-icons**. It
consumes `@game-gpt/vos` for browser-side `checkSource` feedback.

## Run locally

From the repository root:

```bash
pnpm install
pnpm --filter @game-gpt/vos-homepage dev
```

Create and preview a production build with:

```bash
pnpm --filter @game-gpt/vos-homepage check
pnpm --filter @game-gpt/vos-homepage build
pnpm --filter @game-gpt/vos-homepage serve
```

Release static site (CDN — Cloudflare Pages, Netlify, etc.):

```bash
pnpm homepage
```

Outputs land under **`dist/cdn`** (`static` profile). Local dev/SSR uses **`dist/browser`**; WeChat uses **`dist/wechat`**.

CDN host (repo root):

| Field | Value |
|-------|-------|
| Build command | `pnpm homepage` |
| Output directory | `projects/vos.ts/homepage/dist/cdn` |
| Env (optional) | `VMZ_SITE_ORIGIN=https://your.domain` |

Do not enable SPA fallback. `pnpm homepage` runs the `static` profile release emit into `dist/cdn`.

## Content principles

- Lead with the problem VOS solves before listing syntax.
- Keep examples valid against the current language surface.
- Explain VOS as a native schema and operation language, never as a SQL wrapper.
- Keep English and Chinese locale keys aligned when changing visible content.
- State TypeScript checker limitations honestly and point advanced parsing needs to the Rust facade.

## Structure

| Area | Purpose |
|------|---------|
| `src/pages/index.vmz` | Main language narrative (`/`) |
| `src/pages/model.vmz` | Deeper model exploration (`/model`) |
| `src/pages/playground.vmz` | Interactive source checking (`/playground`) |
| `src/lib/samples.ts` | Curated VOS examples |
| `locales/` | English (`en-us`) and Chinese (`zh-hans`) product copy |
| `designs/` | Application-owned tokens and styles |
| `src/components/` | Site chrome and reusable sections |

See the [TypeScript workspace README](../README.md) for package relationships and the
root [VOS README](../../../readme.md) for the project-wide story.
