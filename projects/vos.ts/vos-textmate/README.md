# `@game-gpt/vos-textmate`

The shared TextMate grammar that gives VOS one visual identity across editors and documentation.

This package owns `vos.tmLanguage.json`. The VS Code/Cursor extension receives a synchronized copy, while the VOS
homepage imports the grammar directly for Shiki. Centralizing the grammar prevents subtle differences in how the same
`.vos` source is presented from one surface to another.

## Use with Shiki

```ts
import {createHighlighter} from "shiki";
import {vosLanguage} from "@game-gpt/vos-textmate";

const highlighter = await createHighlighter({
    langs: [vosLanguage],
    themes: ["github-dark"],
});

const html = highlighter.codeToHtml("table User { @@id: uuid }", {
    lang: "vos",
    theme: "github-dark",
});
```

The raw grammar is also exported as `@game-gpt/vos-textmate/vos.tmLanguage.json`.

## Grammar responsibilities

The grammar should recognize stable language forms without trying to become a parser. It covers declarations,
attributes, built-in types, references, literals, paths, methods, projections, comments, and punctuation while remaining
useful for incomplete editor input.

Semantic validity belongs to VOS parsers and host tools. A TextMate rule should improve presentation, not invent a new
interpretation of the language.

## Synchronize the editor extension

After editing `vos.tmLanguage.json`, run:

```bash
pnpm sync-vscode
```

This updates the generated grammar copy in `../vos-on-vscode/syntaxes/`. Do not maintain a separate hand-written grammar
in the extension or homepage.

## Review checklist

- Test both schema declarations and operation programs.
- Include nested references, optional/list wrappers, strings, numbers, and comments.
- Check partially typed source; highlighting should degrade gracefully.
- Confirm Shiki can load the exported language object.
- Sync the extension copy before finishing.

See the [extension README](../vos-on-vscode/README.md) and [TypeScript workspace guide](../README.md) for the consumers
of this package.
