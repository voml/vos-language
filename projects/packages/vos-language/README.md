# `@voml/vos-language`

VOS language tooling for TypeScript hosts: checks and diagnostics for `.vos` source.

Declaration and type modelling live in [`@voml/vos`](../vos) — `v.table`, `v.object`, descriptors and `Infer`. This
package owns the language layer only; it does not re-implement modelling semantics and does not manage database
lifecycle.

Try the interactive playground on the [VOS homepage](https://vos-language.pages.dev/playground).

## Install

```bash
pnpm add @voml/vos-language
```

## Check source

```ts
import { checkSource } from "@voml/vos-language";

const result = checkSource(`
namespace example::identity

service IdentityService {
    login(request: LoginRequest) -> LoginResponse
}
`);

if (result.ok) {
    console.log("The source passed the lightweight checks.");
} else {
    for (const diagnostic of result.diagnostics) {
        console.error(`line ${diagnostic.line}: ${diagnostic.message}`);
    }
}
```

`checkSource` returns a boolean result plus line-oriented diagnostics:

```ts
type VosCheckResult = {
    ok: boolean;
    diagnostics: Array<{
        line: number;
        message: string;
    }>;
};
```

## Scope

Today `checkSource` is a migration-phase narrow checker: it catches empty input, unbalanced braces and brackets, a
missing top-level declaration, and positional service parameters that should be named. It is **not** the canonical VOS
parser and does not produce the Rust AST, catalog identity, or complete semantic diagnostics.

Conformance-grade parsing, resolution, catalog projection and semantic diagnostics come from the Oak backed `oak-vos`
pipeline through the Rust [`vos` facade](../../crates/vos). This package will replace the narrow checker as that
pipeline is wired in; until then the check is deliberately described as narrow rather than dressed up as full language
support.

## Development

```bash
pnpm build
pnpm typecheck
pnpm test
```

See the [TypeScript workspace guide](../README.md) and main [VOS overview](../../../readme.md) for the surrounding
project.
