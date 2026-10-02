import grammar from "./vos.tmLanguage.json" with { type: "json" };

/** TextMate grammar object (VS Code / raw consumers). */
export const vosGrammar = grammar;

/** Language id used by VS Code and Shiki (`lang: 'vos'`). */
export const vosLanguageId = "vos" as const;

/** TextMate scope name. */
export const vosScopeName = "source.vos" as const;

/**
 * Shiki `LanguageRegistration` — same grammar VS Code loads.
 *
 * Prefer {@link createVosHighlighter} so themes stay in sync with homepage/docs.
 */
export const vosLanguage = {
    ...vosGrammar,
    name: vosLanguageId,
    scopeName: vosScopeName,
    aliases: ["vos", "VOS"],
};

export type CreateVosHighlighterOptions = {
    /** Extra Shiki theme ids (default: `vitesse-dark`). */
    themes?: string[];
    /** Extra language ids / registrations beyond VOS. */
    langs?: unknown[];
};

/**
 * Homepage / docs helper: create a Shiki highlighter preloaded with `vos`.
 *
 * Requires peer `shiki` (not bundled here).
 *
 * @example
 * ```ts
 * import { createVosHighlighter } from '@game-gpt/vos-textmate/shiki'
 * const hi = await createVosHighlighter({ themes: ['vitesse-dark'] })
 * hi.codeToHtml(src, { lang: 'vos', theme: 'vitesse-dark' })
 * ```
 */
export async function createVosHighlighter(options: CreateVosHighlighterOptions = {}) {
    const { createHighlighter } = await import("shiki");
    const themes = options.themes?.length ? options.themes : ["vitesse-dark"];
    return createHighlighter({
        langs: [vosLanguage, ...(options.langs ?? [])],
        themes,
    });
}

/** @vmz/plugin-shiki textmate factory alias (same contract as vmz-textmate). */
export const createHighlighter = createVosHighlighter;

export default vosLanguage;
