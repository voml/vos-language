# `vos-contract`

`vos-contract` is the VOS semantic consumer boundary for Oak-owned syntax
trees. It does not lex, parse, recover syntax, or define a second AST parser.
Callers obtain `oak-vos::VosRoot` from Oak and pass it to
`ContractInput::from_oak`.

`ContractInput::project_schema` produces the draft `SchemaProjection` artifact
with a `syntax-projection` envelope stage. It preserves table/class paths,
field attributes, type wrappers and provenance. Named types remain unresolved,
and no durable IDs or schema fingerprint are assigned. This artifact is for
conformance and tooling, not production database or ORM lowering.

`resolve_contract` binds an explicit `IdentityManifest` to the Oak projection
and produces the strict `vos-resolved-contract-v1` artifact. Downstream hosts
must validate its identities, references and fingerprint before lowering.

Run `pnpm conformance:contracts` (or `cargo test -p vos-contract`) for the
projection, identity, resolved-artifact and diagnostic gates. Reviewed fixtures
live in `specifications/fixtures/contracts/`. The basic resolved golden shares
its source and identity manifest with the projection fixture. These gates do
not establish operation/service or downstream execution support.
