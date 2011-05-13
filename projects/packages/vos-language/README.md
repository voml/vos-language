# 🧰 `@voml/vos-language`

> **`.vos` source checks, catalog projection and formatting for TypeScript hosts, running on a Rust WebAssembly core.**

The implementation is the Rust core, not TypeScript. The wasm module is exported from `projects/crates/vos-wasm` and
the TypeScript surface only loads and types it — syntax and semantics stay owned by `oak-vos` and the `vos` facade.
Declaration and type modelling live in
[`@voml/vos`](https://www.npmjs.com/package/@voml/vos).

## 🔍 Check source

Loading the wasm core is asynchronous, so every entry point is asynchronous too:

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

## 🗂️ Project the schema catalog

`catalogSource` returns the resolved `vos-contract` `SchemaProjection` for the same source, so a host reads canonical
paths, field types and attributes without owning a second parser:

```ts
import { catalogSource } from "@voml/vos-language";

const result = await catalogSource("table User { @@id: uuid, name: utf8 }");
if (result.ok) {
    // `catalog` is the Rust projection artifact, handed over as `unknown`.
    console.log(JSON.stringify(result.catalog));
}
```

The catalog shape is owned by the Rust serializer and is not frozen in TypeScript, so it is returned as `unknown` and
the caller narrows it — this package never fabricates a TypeScript shape that could drift from the core.

## 🧹 Format source

Formatting lives on the wasm core as well, so it is reached through the `@voml/vos-language/formatter` subpath:

```ts
import { formatSource } from "@voml/vos-language/formatter";

const result = await formatSource(source, "canonical");
if (result.ok) console.log(result.text);
```

Two modes, mapped one to one onto `vos-format::Mode`:

| Mode | Contract |
| ---- | -------- |
| `preserve` | Replays the author's own spelling byte for byte from the Oak lossless CST. |
| `canonical` | Rewrites the layout of the declarations Oak decomposes, and normalizes static paths to `::`. |

`canonical` is the default. Declarations Oak does not decompose yet are passed through verbatim, and type aliases such
as `vec<3>` keep the author's spelling — rewriting them is a VOS semantic decision, not a formatting one.

## ⚙️ Initialising the wasm core

Every entry point initialises the core on demand. Omitting the argument uses wasm-bindgen's default loader, which
fetches `lib/vos_wasm_bg.wasm` — the browser and Worker path:

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
- **No second implementation.** This package never re-implements VOS parsing, resolution, formatting or diagnostics in
  TypeScript, and never manages database lifecycle.
- **Narrow by design.** For resolution, contract and generator APIs, use the `vos` crate in this repository's Rust
  workspace.

---

`@voml/vos` — [declaration and type modelling](https://www.npmjs.com/package/@voml/vos)
