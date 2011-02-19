import type { LanguageRegistration } from "shiki";
import grammar from "./vos.tmLanguage.json" with { type: "json" };

/**
 * Shared TextMate grammar for VOS (`.vos`).
 *
 * **Single source of truth:** `vos.tmLanguage.json` in this package.
 * - VS Code: `vos-on-vscode` copies/syncs the same file into `syntaxes/`.
 * - Homepage Shiki: load this registration — do not fork a parallel grammar.
 */
export const vosLanguage: LanguageRegistration = {
    ...grammar,
    name: "vos",
    aliases: ["vos", "VOS"],
};

export default vosLanguage;
