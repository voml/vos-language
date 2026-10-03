# VOS Conformance Fixtures

Executable examples of what VOS source means.

The fixture suite is the conformance bridge between Oak, the VOS semantic consumer, and future hosts. It provides
source files and exact expected outputs for normalization, Oak structure, diagnostics, resolved contracts, and catalog
identity.

## How a case is assembled

Each case starts with a stem and may have several companions:

```text
stem.vos                 source under test
stem.normalized.vos      canonical UTF-8, LF-only source without a BOM
stem.ast.json            serialized Oak Builder output for a successful parse
stem.diagnostics.json    ordered Oak/VOS diagnostics
stem.catalog.json        stable field/catalog identity when applicable
stem.kind.json           explicit parser selection in mixed directories
```

A missing AST generally means the source is expected to fail. Diagnostic files always use the same shape, including an
empty `errors` array for successful cases.

## Fixture families

| Directory                      | Focus                                    | Oak/VOS entry                       |
|--------------------------------|------------------------------------------|--------------------------------------|
| [`schema`](./schema)           | Schema and DDL documents                 | Oak VOS frontend                    |
| [`operations`](./operations)   | Expressions and object/method operations | Oak VOS frontend                    |
| [`diagnostics`](./diagnostics) | Invalid or warning-bearing source        | Oak diagnostics                     |

Under `operations/`, the Oak VOS operation surface is expected; elsewhere the Oak VOS schema surface is expected. A
`stem.kind.json` sidecar can make
the choice explicit:

```json
{
    "parser": "document"
}
```

## Golden contracts

### Normalized source

Normalization removes a UTF-8 BOM and converts line endings to LF. Spans in all other outputs are byte offsets into this
normalized source, not the platform-specific input bytes.

### AST JSON

Oak Builder output and VOS resolved artifacts are the golden representations. Source text and spans remain owned by Oak.
Changes are reviewed as language-contract changes rather than formatting noise.

### Diagnostics

Diagnostics use the Oak/VOS structured diagnostic contract:

```json
{
    "errors": [
        {
            "code": "VOS-...",
            "message": "A useful explanation",
            "span": {
                "start": 0,
                "end": 1
            },
            "hint": "A concrete next step"
        }
    ]
}
```

`code` and `hint` may be `null`. Ordering is significant because it matches parser emission order.

### Catalog identity

Schema cases may include `*.catalog.json` snapshots. These capture durable `TypeId`, `FieldId`, virtual field indexes,
and revision state. They complement the source-facing AST: a field's stable identity is not merely its current name or
position.

### Contract projection matrix

`contracts/feature-matrix.json` records each feature across documented,
parsed, projected, resolved, conformance, and downstream-consumed stages. A
true `parsed` or `projected` value is not evidence that durable identity or a
production contract exists.

## Run conformance

From the repo root (`cargo` workspace):

```bash
cargo test -p vos-parser --test conformance
```

The ordinary test path only compares outputs and is safe for CI. Intentional golden updates use:

```bash
VOS_UPDATE_FIXTURES=1 cargo test -p vos-parser --test conformance
```

That environment variable rewrites companion files. Review every generated diff; never treat blessing as a substitute
for understanding a semantic change.

## Add a case

1. Choose the narrowest fixture family.
2. Add a descriptive `stem.vos` file with one primary purpose.
3. Add `stem.kind.json` only when parser selection would otherwise be ambiguous.
4. Run the conformance test in update mode.
5. Inspect normalized source, AST, diagnostics, and catalog output.
6. Re-run without update mode to prove the checked-in contract is stable.

Use VOS-native examples throughout. SQL grammar, SQL-shaped diagnostics, and VOS-to-SQL behavior are outside this
contract.

## What good fixtures look like

A strong fixture is small enough to review, realistic enough to expose interactions, and named after the behavior it
protects. Prefer a new focused case over expanding an unrelated golden until its purpose becomes unclear.

Return to the root [VOS README](../../readme.md) for the project overview.
