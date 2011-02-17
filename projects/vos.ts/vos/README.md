# `@game-gpt/vos`

Fast, dependency-free source checks for VOS in TypeScript and browser environments.

Try the interactive playground on the [VOS homepage](https://vos-language.pages.dev/playground).

The package is the public TypeScript entry point for VOS. Its current API is intentionally small: it catches empty
input, unbalanced braces and brackets, missing top-level declarations, and positional service parameters that should be
named.

## Install

```bash
pnpm add @game-gpt/vos
```

## Check source

```ts
import {checkSource} from "@game-gpt/vos";

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

This package is useful for immediate feedback in documentation, playgrounds, and lightweight tooling. It is **not yet
the canonical VOS parser** and does not currently produce the Rust AST, catalog identity, or complete semantic
diagnostics.

Use the Rust [`vos` facade](../../vos.rs) for conformance-grade parsing. The TypeScript surface will grow only when
behavior can be explicit and tested rather than approximated behind a larger-looking API.

## Development

```bash
pnpm build
pnpm typecheck
pnpm test
```

See the [TypeScript workspace guide](../README.md) and main [VOS overview](../../../readme.md) for the surrounding
project.
