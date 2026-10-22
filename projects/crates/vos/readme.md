# `vos`

The stable Rust facade for Virtual Object Schema.

Use this crate when building a database host, compiler, generator, CLI, or developer tool that needs to understand
`.vos` source. Oak owns lexing and parsing. This facade exposes Oak-backed contract input and VOS semantic contracts.

## Oak-backed contract input

New integrations should begin with `vos::parse_oak`, which invokes Oak and returns
`vos::ContractInput`. VOS semantic consumers must operate on that Oak output and must not add another
parser or reparse the source. After an explicit identity manifest is available, call `vos::resolve_contract` and pass
the resulting `vos::ResolvedContract` to database or ORM adapters.

## Public modules

- `vos::types` provides foundational semantic models and strict artifact readers from `vos-types`, without an Oak dependency.
- `vos::parse_oak` is the Oak-owned frontend entry for contract consumers.
- `vos::resolve_contract` is the resolved semantic contract entry for downstream hosts.
- Oak owns the lexer, parser, CST, Builder AST, spans, and recovery. This crate owns no parser, AST or CST of its own.
- `vos::generator` renders artifacts through Dejavu templates.

Expression and operation programs, inspect policy checks, the field-identity catalog and the miette diagnostic
reporters were removed with the `vos-ast`, `vos-parser` and `vos-inspect` crates. They return when Oak's VOS frontend
covers expression, operation, `micro` and `macro` grammar and the consumer layer is rebuilt on that output.

- `vos::uuid()` generates **UUID v7 only** (VOS builtin `uuid()`). Do not use v4 for PK columns — see
  [`../../../documentation`](../../../documentation/uuid-v7.md) (page-split rationale).

## Integration rule

Applications should depend on this facade rather than internal syntax crates. Hosts must consume Oak-backed VOS
contracts and must not parse or traverse a private VOS AST.

See the [Rust workspace guide](../README.md) for architecture, conformance, and development commands, or the
root [VOS overview](../../../readme.md) for the language story.
