/**
 * Serialize VOS descriptors back into a `.vos` source document.
 *
 * This is the reverse bridge of the `v` type builder: callers author
 * descriptors with `v.table()`, feed a list of them through `toSource()`
 * and get the canonical `.vos` text that the Rust engine's
 * `ensure_schema(document: &str)` expects.
 *
 * Coverage per descriptor feature:
 *   - builtin scalar kinds (i8…datetimeUtc + bytes/file/vector)
 *   - wrapper kinds (optional T?, list [T], ref &T)
 *   - bracket attributes ([primary] [unique] [tag(x)] …) — rendered as prefix
 *   - named declarations (table / class) + anonymous object
 *   - field ordering is preserved from the original descriptor object keys
 *
 * The serializer deliberately does **not** emit `namespace`, `enums`, `flags`,
 * `obsolete`, or trailing micro declarations — the descriptor surface
 * (`@voml/vos`) simply doesn't model them. If any of these are needed, build
 * the `.vos` text by hand and pass it through `ensureSchema` directly.
 */

import {
    type AnyTypeDescriptor,
    type FieldDescriptor,
    type RefDescriptor,
    type ScalarKind,
} from "./descriptor.js";
import {
    type TableDescriptor,
    type ClassDescriptor,
    type ObjectDescriptor,
    type NamedItemDescriptor,
} from "./items.js";

/**
 * Serialize one or more named declarations (table / class) into a `.vos`
 * source document. Anonymous `object` descriptors are skipped at the top
 * level — they don't belong in a database schema file.
 *
 * The output is pure VOS source text. Pass it directly to
 * `@yydb/yydb`'s `Database.ensureSchema` or the Rust engine.
 */
export function toSource(
    ...items: readonly (TableDescriptor | ClassDescriptor)[]
): string {
    const out: string[] = [];
    for (const item of items) {
        out.push(renderDeclaration(item));
    }
    // Ensure exactly one trailing newline.
    while (out.length > 0 && out[out.length - 1] === "") {
        out.pop();
    }
    return out.join("\n\n") + "\n";
}

/** Serialize a single item descriptor. */
function renderDeclaration(item: TableDescriptor | ClassDescriptor): string {
    const lines: string[] = [];
    lines.push(`${item.kind} ${item.name} {`);
    const entries = Object.values(item.fields);
    for (const [idx, field] of entries.entries()) {
        let line = renderField(field as FieldDescriptor);
        if (idx < entries.length - 1) {
            line += ",";
        }
        lines.push(`    ${line}`);
    }
    lines.push("}");
    return lines.join("\n");
}

/** Serialize one named field: `[attrs] name: type_expr [= default]`. */
function renderField(field: FieldDescriptor): string {
    const attrs = renderAttrPrefix(field.attrs);
    return `${attrs}${field.name}: ${renderType(field)}`;
}

/** Render bracket attributes as a prefix, e.g. `[primary] ` or `[tag(1)] `. */
function renderAttrPrefix(attrs: readonly { readonly name: string; readonly args: readonly string[] }[]): string {
    if (attrs.length === 0) return "";
    return attrs.map((a) => {
        if (a.args.length === 0) return `[${a.name}]`;
        return `[${a.name}(${a.args.join(",")})]`;
    }).join(" ") + " ";
}

/** Render a scalar or wrapper descriptor to its VOS type expression. */
function renderType(descriptor: AnyTypeDescriptor): string {
    switch (descriptor.kind as ScalarKind) {
        case "optional": {
            const inner = descriptor.inner;
            if (!inner) {
                throw new Error("optional descriptor missing .inner");
            }
            return `${renderType(inner)}?`;
        }
        case "list": {
            const inner = descriptor.inner;
            if (!inner) {
                throw new Error("list descriptor missing .inner");
            }
            return `[${renderType(inner)}]`;
        }
        case "ref": {
            const target = (descriptor as RefDescriptor).target;
            if (!target) {
                throw new Error("ref descriptor missing .target");
            }
            return `&${target.name}`;
        }
        case "vector": {
            const dim = (descriptor as { readonly dim?: number }).dim;
            if (dim === undefined) {
                throw new Error("vector descriptor missing .dim");
            }
            return `vector<${dim}>`;
        }
        default:
            // Plain builtin scalar kind (i8 / utf8 / uuid / bytes / file …)
            return descriptor.kind;
    }
}
