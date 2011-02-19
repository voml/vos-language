/**
 * Public TypeScript surface for VOS.
 *
 * `import { … } from "@game-gpt/vos"`
 */

export type VosDiagnostic = {
    line: number;
    message: string;
};

export type VosCheckResult = {
    ok: boolean;
    diagnostics: VosDiagnostic[];
};

const TOP_LEVEL = /\b(namespace|using|table|class|enums|flags|union|service|const|obsolete)\b/;

/** Lightweight source sanity check until the real parser lands. */
export function checkSource(source: string): VosCheckResult {
    const diagnostics: VosDiagnostic[] = [];
    const lines = source.split(/\r?\n/);

    if (!source.trim()) {
        diagnostics.push({ line: 1, message: "source is empty" });
        return { ok: false, diagnostics };
    }

    let depth = 0;
    let bracket = 0;
    lines.forEach((line, index) => {
        const code = line.replace(/#.*$/, "");
        for (const ch of code) {
            if (ch === "{") depth += 1;
            if (ch === "}") depth -= 1;
            if (ch === "[") bracket += 1;
            if (ch === "]") bracket -= 1;
            if (depth < 0) {
                diagnostics.push({
                    line: index + 1,
                    message: "unexpected closing brace '}'",
                });
                depth = 0;
            }
            if (bracket < 0) {
                diagnostics.push({
                    line: index + 1,
                    message: "unexpected closing bracket ']'",
                });
                bracket = 0;
            }
        }
    });

    if (depth > 0) {
        diagnostics.push({
            line: lines.length,
            message: `unclosed '{': ${depth} remaining`,
        });
    }
    if (bracket > 0) {
        diagnostics.push({
            line: lines.length,
            message: `unclosed '[': ${bracket} remaining`,
        });
    }

    if (!TOP_LEVEL.test(source)) {
        diagnostics.push({
            line: 1,
            message: "expected a top-level item: namespace, using, table, class, enums, flags, union, service, const, or obsolete",
        });
    }

    // Catch positional service parameters (must be named).
    const positional = source.match(/\b([A-Za-z_][A-Za-z0-9_]*)\s*\(\s*([A-Za-z_][A-Za-z0-9_:]*)\s*\)\s*->/);
    if (positional) {
        const method = positional[1]!;
        const ty = positional[2]!;
        const line = source.slice(0, positional.index ?? 0).split(/\r?\n/).length || 1;
        diagnostics.push({
            line,
            message: `service method \`${method}\` requires a named parameter; use \`request: ${ty}\``,
        });
    }

    return { ok: diagnostics.length === 0, diagnostics };
}
