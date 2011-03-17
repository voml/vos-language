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
