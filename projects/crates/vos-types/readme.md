# `vos-types`

Foundational VOS semantic models, identity/artifact data structures, strict JSON
encoding and validation, and deterministic semantic fingerprints. This crate has
no dependency on Oak or on the VOS processing facade.

Source parsing, Oak projection, name resolution and identity evolution are
provided by `vos`. Applications use that facade and access models through
`vos::types` or its root exports. Artifact formats retain their existing version
identifiers independently of Rust crate names.

Run `pnpm conformance:contracts` for the model and semantic-processing gates.
Reviewed fixtures live in `../../../documentation`. These gates do
not establish complete operation/service or downstream execution support.
