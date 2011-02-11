# VOS for VS Code and Cursor

Make `.vos` files feel at home in VS Code and Cursor with language registration, bracket and comment behavior, and
syntax highlighting for the VOS declaration and operation vocabulary.

The extension is intentionally focused. It provides a reliable editing surface today while parser diagnostics and host
checks remain owned by VOS implementations rather than duplicated in editor-specific code.

## What you get

- `.vos` file recognition
- VOS syntax highlighting
- matching brackets and braces
- line-comment configuration
- the same token grammar used by the VOS homepage

Language-server features such as completion, navigation, and canonical diagnostics are not included yet.

## Develop the extension

Open this directory in VS Code or Cursor and launch an Extension Development Host. You can also install the unpacked
extension directory for local testing:

```bash
code --install-extension ./projects/vos.ts/vos-on-vscode
```

## One grammar, multiple surfaces

The editable source of truth is [`../vos-textmate/vos.tmLanguage.json`](../vos-textmate/vos.tmLanguage.json). The file
under
`syntaxes/` is synchronized for extension packaging.

After changing the source grammar, run:

```bash
pnpm --filter vos-on-vscode sync-grammar
```

Do not hand-edit `syntaxes/vos.tmLanguage.json`; the next sync will replace it. One grammar ensures that a VOS example
is highlighted consistently in the editor and in Shiki-powered documentation.

## Test a grammar change

Use examples that cover declarations, field attributes, references, operation chains, projections, and comments. The
shared [conformance fixtures](../../../specifications/fixtures/README.md) are a useful source of realistic syntax, but
highlighting must remain tolerant of partially written code.

See the [TypeScript workspace README](../README.md) for the complete tooling map.
