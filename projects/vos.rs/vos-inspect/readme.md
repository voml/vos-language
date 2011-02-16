# `vos-inspect`

Configurable, post-parse inspection for VOS programs.

Inspection is deliberately separate from parsing. The parser decides whether source is valid; the inspect engine lets a
host apply project policy at `off`, `allow`, `warn`, or `deny` without changing the underlying language semantics.

## Current capabilities

The engine provides typed rule identifiers, levels, configuration, findings, reports, and association-analysis helpers.
Its first rule can prefer explicit `.load(...)` calls while preserving the language guarantee that reference use sites
remain valid without them.

This distinction matters: a team may want explicit loads for reviewability or performance policy, but VOS must not turn
that preference into a universal syntax requirement.

## Typical flow

1. Parse and baseline-check the schema or program.
2. Build an `InspectConfig` for the host or project.
3. Run `InspectEngine` against valid typed input.
4. Present findings according to their configured levels.

## Dependency boundary

Applications should not depend on this crate directly. Use `vos::inspect` from the public [`vos`](../vos) facade so
inspection remains part of one coherent host API.

See the [Rust workspace README](../README.md) for architecture and verification commands.
