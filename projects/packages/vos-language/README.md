# 🧰 `@voml/vos-language`

> **`.vos` source checks and diagnostics for TypeScript hosts, running on a Rust WebAssembly core.**

The implementation is the Rust core, not TypeScript. The wasm module is exported from `projects/crates/vos-wasm` and
the TypeScript surface only loads and types it — syntax and semantics stay owned by `oak-vos` and the `vos` facade.
Declaration and type modelling live in
[`@voml/vos`](https://www.npmjs.com/package/@voml/vos).

## 🔍 Check source

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

## ⚙️ Initialising the wasm core

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

## 🚧 Scope

- **Reports what the Rust core reports.** A diagnostic that cannot be located to a line is returned without a line
  number rather than guessed.
- **No second implementation.** This package never re-implements VOS parsing, resolution or diagnostics in TypeScript,
  and never manages database lifecycle.
- **Narrow by design.** For the full contract, catalog and generator APIs, use the `vos` crate in this repository's
  Rust workspace.

---

`@voml/vos` — [declaration and type modelling](https://www.npmjs.com/package/@voml/vos)
