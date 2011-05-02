# `@voml/vos-language`

VOS language tooling for TypeScript hosts: checks and diagnostics for `.vos` source.

The implementation is the **Rust WebAssembly core**, not TypeScript. `lib/` holds the wasm-bindgen output built from
`projects/crates/vos-wasm` by `scripts/build-wasm.mjs`, and `src/index.ts` only loads and types it. Source checks are
never re-implemented on the TypeScript side: syntax and semantics stay owned by `oak-vos` and the `vos` facade.

Declaration and type modelling live in [`@voml/vos`](../vos) — `v.table`, `v.object`, descriptors and `Infer`. This
package owns the language layer only.

Try the interactive playground on the [VOS homepage](https://vos-language.pages.dev/playground).

## Install

```bash
pnpm add @voml/vos-language
```

## Check source

Loading the wasm core is asynchronous, so the check is asynchronous too:

```ts
import { checkSource } from "@voml/vos-language";

const result = await checkSource(`
table User {
    @@id: uuid,
    name: utf8,
}
`);

if (result.ok) {
    console.log("The schema passed the VOS checks.");
} else {
    for (const diagnostic of result.diagnostics) {
        console.error(diagnostic.line != null ? `L${diagnostic.line}: ${diagnostic.message}` : diagnostic.message);
    }
}
```

`checkSource` does not throw: parse, projection and semantic failures are reported through the diagnostics.

```ts
type VosCheckResult = {
    ok: boolean;
    diagnostics: Array<{
        line?: number; // present only when the host can locate the line
        message: string;
    }>;
};
```

## Initialising the wasm core

`checkSource` initialises the core on demand. Omitting the argument uses wasm-bindgen's default loader, which fetches
`lib/vos_wasm_bg.wasm` — the browser and Worker path:

```ts
import { initWasm } from "@voml/vos-language";

await initWasm();
```

Node cannot `fetch` a `file:` URL, so a Node host passes the bytes it read itself:

```ts
import { readFileSync } from "node:fs";
import { checkSource, initWasm } from "@voml/vos-language";

await initWasm(readFileSync(new URL("../lib/vos_wasm_bg.wasm", import.meta.url)));
const result = await checkSource(source);
```

`initWasm` is idempotent: repeated calls reuse the first load.

## Build

```bash
pnpm build            # builds projects/crates/vos-wasm and stages lib/
pnpm typecheck
pnpm test
```

`lib/` is generated output and is not committed; `typecheck`, `test` and `build` all rebuild it first when it is
missing or stale. The build needs the `wasm32-unknown-unknown` target and a `wasm-bindgen` CLI matching the version in
`Cargo.lock` (the build script installs it when it differs).

Because the repository-local `.cargo/config.toml` patches `oak-vos` to a sibling checkout, the build is driven from
outside the repository so the `oak-vos` revision pinned in `Cargo.toml` is used instead.

## Scope

This package reports what the Rust core reports. Diagnostics that cannot be located to a line are returned without a
line number rather than guessed. Use the Rust [`vos` facade](../../crates/vos) directly for the full contract,
catalog and generator APIs.

See the [TypeScript workspace guide](../README.md) and main [VOS overview](../../../readme.md) for the surrounding
project.
