# `vos-parser`

The internal VOS semantic lowering and baseline checker. Oak owns the source
syntax frontend and this crate lowers its accepted VOS surface into the
reference VOS AST before semantic checking.

It first validates the source through `oak-vos`, then lowers normalized `.vos` source into the typed structures from `vos-ast`, emits stable diagnostics with byte spans,
and adapts those diagnostics to source-aware miette reports for terminals and host applications.

## Entry points

- `parse_document` parses schema and DDL documents.
- `parse_program` parses expressions and operation programs.
- `normalize_source` produces the canonical UTF-8, LF-only source used by conformance.
- `check` validates a parsed document.
- `report_diagnostic` and `report_diagnostics` attach file names and source text to failures.

Oak currently provides the structural syntax gate. The legacy semantic lowering remains in this crate until the full
VOS Builder AST and conformance migration is complete. Optional style or policy decisions belong to `vos-inspect`, which runs
after this baseline succeeds.

## Conformance

The parser owns the shared golden runner:

```bash
cargo test -p vos-parser --test conformance
```

Fixtures cover schema documents, operation programs, illegal inputs, normalized source, AST serialization, diagnostics,
and catalog output. Read the [fixture guide](../../../specifications/fixtures/README.md) before updating expected files.

## Dependency boundary

Hosts should depend on [`vos`](../vos) and call the parser through `vos::parser`. This crate is not published
independently.

See the [Rust workspace README](../README.md) for full verification commands.
