/**
 * Canonical VOS source emission for declaration descriptors.
 *
 * A descriptor reaches a host either as a canonical artifact or as canonical VOS
 * source that the Oak backed pipeline parses and checks. This module implements
 * the second path: it renders `table` / `class` declarations as canonical DDL
 * text following `vos/documentation/ddl.md`.
 *
 * Anonymous `object` descriptors are projections and parameters, not schema
 * declarations, so they cannot be emitted here.
 */

import { type AnyTypeDescriptor, type Attribute, type BuiltinKind, type FieldDescriptor } from "./descriptor.js";
import type { FieldMap, NamedItemDescriptor, NamedItemKind } from "./items.js";

const BUILTIN_SPELLING: Record<BuiltinKind, string> = {
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

/** Declaration that can be rendered as canonical VOS source. */
export type EmittableItem = NamedItemDescriptor<NamedItemKind, string, FieldMap>;

/**
 * Render one declaration, or a list of declarations, as canonical VOS source.
 *
 * Declarations are separated by a blank line and the result ends with a
 * trailing newline.
 */
export function emitSource(input: EmittableItem | readonly EmittableItem[]): string {
    const items = Array.isArray(input) ? (input as readonly EmittableItem[]) : [input as EmittableItem];
    if (items.length === 0) {
        throw new Error("emitSource: at least one declaration is required");
    }
    return `${items.map(emitItem).join("\n\n")}\n`;
}

function emitItem(item: EmittableItem): string {
    if (item.kind !== "table" && item.kind !== "class") {
        throw new Error(`emitSource: \`${item.kind}\` is not a schema declaration, use \`v.class\` for a named non-persistent declaration`);
    }
    const lines = Object.values(item.fields).map(emitField);
    return `${item.kind} ${item.name} {\n${lines.map((line) => `${line}\n`).join("")}}`;
}

function emitField(field: FieldDescriptor): string {
    return `    ${emitAttributes(field)}${field.name}: ${emitType(field)},`;
}

function emitAttributes(field: FieldDescriptor): string {
    const attrs = field.attrs;
    if (attrs.length === 1 && attrs[0]?.name === "primary") {
        return "@@";
    }
    if (attrs.length === 1 && attrs[0]?.name === "unique") {
        return "@";
    }
    if (attrs.length === 0) {
        return "";
    }
    return `[${attrs.map(emitAttribute).join(", ")}] `;
}

/** Attribute arguments are rendered verbatim; callers pass VOS literal spelling. */
function emitAttribute(attribute: Attribute): string {
    return attribute.args.length === 0 ? attribute.name : `${attribute.name}(${attribute.args.join(", ")})`;
}

function emitType(descriptor: AnyTypeDescriptor): string {
    switch (descriptor.kind) {
        case "optional":
            return `${emitWrapped(descriptor)}?`;
        case "list":
            return `[${emitWrapped(descriptor)}]`;
        case "ref": {
            if (!descriptor.target) {
                throw new Error("emitSource: a `ref` descriptor is missing its target declaration");
            }
            return `&${descriptor.target.name}`;
        }
        case "vector": {
            if (descriptor.dim === undefined) {
                throw new Error("emitSource: a `vector` descriptor is missing its dimension");
            }
            return `vector<${descriptor.dim}>`;
        }
        default:
            return BUILTIN_SPELLING[descriptor.kind];
    }
}

function emitWrapped(descriptor: AnyTypeDescriptor): string {
    if (!descriptor.inner) {
        throw new Error(`emitSource: a \`${descriptor.kind}\` descriptor is missing its inner type`);
    }
    return emitType(descriptor.inner);
}
