# `vos-wasm`

WebAssembly export surface for the VOS language toolchain.

`@voml/vos-language` ships the wasm-bindgen output built from this crate under its `lib/` directory.
The TypeScript wrapper in that package loads the artifact and exposes the author-facing API.

The exported surface is a narrow JSON ABI (`check(source)`, `format(source, mode)` and
`catalog(source)`, each returning a string) so the host does not depend on generated JavaScript
object shapes.

Syntax ownership stays with `oak-vos`; this crate owns no parser, CST or AST.

## Build

The crate is built to `wasm32-unknown-unknown` and staged into the npm package `lib/` by
`scripts/build-wasm.mjs` at the repository root.
