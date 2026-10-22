/**
 * Serialize VOS descriptors back into a `.vos` source document.
 *
 * Two public entry points that share the same renderer:
 *
 *   - `emitSource(input)` — accepts a single declaration or an array of them,
 *     throws on empty input. Mirrors the published npm 0.0.1 API surface so
 *     `import { emitSource, v } from "@voml/vos"` works everywhere the crate
 *     is consumed (yydb, light-db, vos-language e2e tests, user code).
 *
 *   - `toSource(...items)` — variadic alias, drops empty input silently.
 *     Convenient in the `emitSource(User, Account, LoginRequest)` call style.
 *
 * Formatting follows the Oak-backed `oak-vos` parser's canonical spellings:
 *
 *   - `@@field` / `@field` sugar for fields whose attrs are exactly `primary`
 *     or `unique` (single attribute). Any other attribute set falls back to
 *     bracket form: `[primary, tag(1)] field`.
 *   - `DateTime<UTC>` spellings for builtin scalars that have distinct VOS
 *     surface forms (the descriptor kind is `datetimeUtc`, the source spelling
 *     is `DateTime<UTC>`).
 *   - Trailing commas on every field line (same as the fixtures the Rust
 *     engine validates against).
 *   - `table` / `class` only — anonymous `object` descriptors are rejected
 *     at the top level because they never belong in a database schema.
 *
 * Coverage per descriptor feature:
 *   - builtin scalar kinds (i8…datetimeUtc + bytes/file)
 *   - wrapper kinds (optional T?, list [T], ref &T)
 *   - `vector<N>` fixed-dimension scalar
 *   - single-attribute `primary` / `unique` → `@@` / `@` sugar
 *   - multi-attribute / other attribute names → bracket prefix
 *
 * Deliberately not modeled by the descriptor surface and therefore not
 * emitted: `namespace`, `enum`, `flags`, `obsolete`, default value `= expr`,
 * trailing micro declarations. Callers build those by hand and concatenate.
 */

import {
    type AnyTypeDescriptor,
    type Attribute,
    type FieldDescriptor,
    type RefDescriptor,
} from "./descriptor.js";
import {
    type ClassDescriptor,
    type FieldMap,
    type NamedItemDescriptor,
    type NamedItemKind,
    type TableDescriptor,
} from "./items.js";

/**
 * Input accepted by `emitSource`: any named declaration. Alias for
 * `NamedItemDescriptor<NamedItemKind, string, FieldMap>`.
 */
export type EmittableItem = NamedItemDescriptor<NamedItemKind, string, FieldMap>;

/**
 * Builtin kind → VOS source spelling map. Scalar kinds that have exactly
 * one matching spelling (i8, u8, utf8, uuid, bytes, file …) just use their
 * own name as the source form. The special-case entries below are the ones
 * where the descriptor kind differs from the VOS surface form.
 */
const BUILTIN_SPELLING: Readonly<Record<string, string>> = {
    i8: "i8",
    i16: "i16",
    i32: "i32",
    i64: "i64",
    u8: "u8",
    u16: "u16",
    u32: "u32",
    u64: "u64",
    f32: "f32",
    f64: "f64",
    bool: "bool",
    utf8: "utf8",
    utf16: "utf16",
    uuid: "uuid",
    decimal: "decimal",
    date: "date",
    time: "time",
    datetimeUtc: "DateTime<UTC>",
    bytes: "bytes",
    file: "file",
    vector: "vector",
};

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/**
 * Render one declaration or an array of declarations into a canonical VOS
 * source document. Accepts single-value or array (mirrors the published API).
 * Throws if `items` is empty.
 */
export function emitSource(input: EmittableItem | readonly EmittableItem[]): string {
    const items = Array.isArray(input) ? input : [input];
    if (items.length === 0) {
        throw new Error("emitSource: at least one declaration is required");
    }
    return `${items.map(renderItem).join("\n\n")}\n`;
}

/**
 * Variadic convenience form of `emitSource`. Drops empty input silently
 * (returns ""), never throws.
 *
 * ```ts
 * // same
 * emitSource([User, Account]);
 * toSource(User, Account);
 * ```
 */
export function toSource(...items: readonly EmittableItem[]): string {
    if (items.length === 0) return "";
    return emitSource(items);
}

// ---------------------------------------------------------------------------
// Internal rendering
// ---------------------------------------------------------------------------

/** Render one named declaration (`table` / `class`). */
function renderItem(item: EmittableItem): string {
    if (item.kind !== "table" && item.kind !== "class") {
        throw new Error(
            `emitSource: \`${item.kind}\` is not a schema declaration — use \`v.class\` for a named non-persistent declaration`,
        );
    }
    const lines = Object.values(item.fields).map(renderField);
    return `${item.kind} ${item.name} {\n${lines.map((l) => `    ${l}`).join("\n")}\n}`;
}

/** Render one field line including attributes and a trailing comma. */
function renderField(field: FieldDescriptor): string {
    const attrPrefix = renderAttributes(field.attrs);
    return `${attrPrefix}${field.name}: ${renderType(field)},`;
}

/**
 * Render field attributes as a prefix.
 *
 *   - No attributes: empty string.
 *   - Single `primary` → `@@`.
 *   - Single `unique`  → `@`.
 *   - Any other single attribute or multiple attributes → bracket form.
 */
function renderAttributes(attrs: readonly Attribute[]): string {
    if (attrs.length === 0) return "";
    if (attrs.length === 1) {
        if (attrs[0]!.name === "primary") return "@@";
        if (attrs[0]!.name === "unique") return "@";
    }
    return `[${attrs.map(renderAttribute).join(", ")}] `;
}

/** Render one attribute: bare name when no args, else `name(arg1, arg2)`. */
function renderAttribute(attr: Attribute): string {
    return attr.args.length === 0 ? attr.name : `${attr.name}(${attr.args.join(", ")})`;
}

/** Render a type descriptor to its VOS source expression. */
function renderType(descriptor: AnyTypeDescriptor): string {
    switch (descriptor.kind) {
        case "optional":
            return `${renderWrapped(descriptor)}?`;
        case "list":
            return `[${renderWrapped(descriptor)}]`;
        case "ref": {
            const target = (descriptor as RefDescriptor).target;
            if (!target) {
                throw new Error("emitSource: a `ref` descriptor is missing its target declaration");
            }
            return `&${target.name}`;
        }
        case "vector": {
            const dim = (descriptor as { readonly dim?: number }).dim;
            if (dim === undefined) {
                throw new Error("emitSource: a `vector` descriptor is missing its dimension");
            }
            return `vector<${dim}>`;
        }
        default:
            // Plain builtin scalar kind — map through BUILTIN_SPELLING to pick
            // up the `datetimeUtc → DateTime<UTC>` spelling and any future
            // special cases we add to Oak's surface.
            return BUILTIN_SPELLING[descriptor.kind] ?? descriptor.kind;
    }
}

/** Render an `optional` or `list`'s inner type. */
function renderWrapped(descriptor: AnyTypeDescriptor): string {
    const inner = (descriptor as { readonly inner?: AnyTypeDescriptor }).inner;
    if (!inner) {
        throw new Error(`emitSource: a \`${descriptor.kind}\` descriptor is missing its inner type`);
    }
    return renderType(inner);
}
