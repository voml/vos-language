/**
 * VOS declaration descriptors: `table`, `class` and structural `object`, plus
 * the `&T` reference wrapper.
 *
 * Field names are addressed by name, never by declaration order
 * (`vos/documentation/semantics.md`). A field descriptor is therefore a scalar
 * descriptor that also carries its VOS field name, and it is reachable both
 * through `descriptor.fields.<name>` and directly as `descriptor.<name>`.
 */

import {
    type AnyTypeDescriptor,
    type FieldDescriptor,
    type Infer,
    makeDescriptor,
    PHANTOM,
    type RefDescriptor,
    type ReferenceTarget,
    withName,
} from "./descriptor.js";
import type { ReferenceKey } from "./values.js";

/** Field literal accepted by `v.table`, `v.class` and `v.object`. */
export type FieldMap = Record<string, AnyTypeDescriptor>;

/** Author-side row shape of a field map. */
export type RowOf<F extends FieldMap> = { [K in keyof F & string]: Infer<F[K]> };

/** Field map with every descriptor named by its key. */
export type NamedFieldMap<F extends FieldMap> = { readonly [K in keyof F & string]: FieldDescriptor<K, Infer<F[K]>> };

/** Declaration kinds that own a durable or named identity. */
export type NamedItemKind = "table" | "class";

/** Declaration kinds this module can describe. */
export type ItemKind = NamedItemKind | "object";

/** `table` / `class` declaration descriptor. */
export interface NamedItemDescriptor<Kind extends NamedItemKind, N extends string, F extends FieldMap> {
    readonly kind: Kind;
    readonly name: N;
    readonly fields: NamedFieldMap<F>;
    readonly [PHANTOM]: RowOf<F>;
}

/** Anonymous structural object descriptor, used for parameters and projections. */
export interface StructuralDescriptor<F extends FieldMap> {
    readonly kind: "object";
    readonly fields: NamedFieldMap<F>;
    readonly [PHANTOM]: RowOf<F>;
}

/** Persistent `table` descriptor. */
export type TableDescriptor<N extends string = string, F extends FieldMap = FieldMap> = NamedItemDescriptor<"table", N, F> & NamedFieldMap<F>;

/** Non-persistent named `class` descriptor. */
export type ClassDescriptor<N extends string = string, F extends FieldMap = FieldMap> = NamedItemDescriptor<"class", N, F> & NamedFieldMap<F>;

/** Anonymous structural object descriptor with direct field access. */
export type ObjectDescriptor<F extends FieldMap = FieldMap> = StructuralDescriptor<F> & NamedFieldMap<F>;

const RESERVED_FIELD_NAMES: readonly string[] = ["kind", "name", "fields"];
const IDENTIFIER = /^[A-Za-z_][A-Za-z0-9_]*$/;

/** Declare a persistent `table`. */
export function makeTable<N extends string, F extends FieldMap>(name: N, fields: F): TableDescriptor<N, F> {
    return makeItem("table", name, fields) as TableDescriptor<N, F>;
}

/** Declare a non-persistent named `class`. */
export function makeClass<N extends string, F extends FieldMap>(name: N, fields: F): ClassDescriptor<N, F> {
    return makeItem("class", name, fields) as ClassDescriptor<N, F>;
}

/** Declare an anonymous structural object for parameters and projections. */
export function makeObject<F extends FieldMap>(fields: F): ObjectDescriptor<F> {
    return makeItem("object", undefined, fields) as ObjectDescriptor<F>;
}

/**
 * Declare a `&T` reference. `&T` stores the target's primary key only; it never
 * embeds the target record (`vos/documentation/semantics.md`).
 */
export function makeRef<T extends ReferenceTarget>(target: T): RefDescriptor<T["name"]> {
    assertReferenceTarget(target);
    const targetRef: ReferenceTarget = { kind: target.kind, name: target.name };
    return makeDescriptor<"ref", ReferenceKey>("ref", { target: targetRef }) as unknown as RefDescriptor<T["name"]>;
}

function makeItem(kind: ItemKind, name: string | undefined, fields: FieldMap): unknown {
    if (name !== undefined) {
        assertIdentifier(name, `v.${kind} name`);
    }
    const fieldNames = Object.keys(fields);
    for (const fieldName of fieldNames) {
        assertIdentifier(fieldName, `v.${kind} field name`);
        if (RESERVED_FIELD_NAMES.includes(fieldName)) {
            throw new Error(`v.${kind} field name \`${fieldName}\` is reserved`);
        }
    }

    const namedFields: Record<string, FieldDescriptor> = {};
    for (const fieldName of fieldNames) {
        namedFields[fieldName] = withName(fields[fieldName]!, fieldName);
    }

    const descriptor: Record<string, unknown> = name === undefined ? { kind, fields: namedFields } : { name, kind, fields: namedFields };

    for (const fieldName of fieldNames) {
        Object.defineProperty(descriptor, fieldName, {
            value: namedFields[fieldName],
            enumerable: false,
            writable: false,
            configurable: false,
        });
    }
    return descriptor;
}

function assertIdentifier(value: string, context: string): void {
    if (!IDENTIFIER.test(value)) {
        throw new Error(`${context}: \`${value}\` is not a valid VOS identifier`);
    }
}

function assertReferenceTarget(target: ReferenceTarget): void {
    if (target.kind !== "table" && target.kind !== "class") {
        throw new Error(`v.ref(target): a reference must target a named \`table\` or \`class\`, got \`${target.kind}\``);
    }
    assertIdentifier(target.name, "v.ref(target) name");
}
