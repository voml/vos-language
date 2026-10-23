/**
 * VOS type descriptor protocol.
 *
 * A descriptor carries plain, serializable data (`kind`, `attrs`, …) plus a few
 * builder helpers. It is **not** a stable database ABI: a descriptor must be
 * compiled into a canonical VOS artifact before any host consumes it.
 *
 * The protocol mirrors the frozen VOS type algebra in
 * `vos/documentation/builtins.md`. Builtin names are case-sensitive and the
 * public surface must not offer a wide `string()` / `number()` / `json()`.
 */

import type { SafeParseResult, ValidationIssue } from "./validate.js";
import type { ReferenceKey } from "./values.js";
import { safeParse as validateSafeParse, parse as validateParse } from "./validate.js";

/** Phantom carrier of the author-side value type. Never present at runtime. */
export declare const PHANTOM: unique symbol;

/** Open bracket attribute, for example `[primary]` or `[tag(1)]`. */
export interface Attribute {
    readonly name: string;
    readonly args: readonly string[];
}

/** Builtin scalar names, spelled exactly as VOS spells them. */
export type BuiltinKind =
    | "i8"
    | "i16"
    | "i32"
    | "i64"
    | "u8"
    | "u16"
    | "u32"
    | "u64"
    | "f32"
    | "f64"
    | "bool"
    | "utf8"
    | "utf16"
    | "uuid"
    | "decimal"
    | "date"
    | "time"
    | "datetimeUtc"
    | "bytes"
    | "file"
    | "vector";

/** Wrapper kinds. VOS wrappers are not builtins. */
export type WrapperKind = "optional" | "list" | "ref";

/** Every scalar descriptor kind: builtin or wrapper. */
export type ScalarKind = BuiltinKind | WrapperKind;

/** Named declaration a `&T` reference can point at. */
export interface ReferenceTarget {
    readonly kind: string;
    readonly name: string;
}

/** A VOS scalar type descriptor. */
export interface TypeDescriptor<K extends ScalarKind = ScalarKind, V = unknown> {
    readonly kind: K;
    /** Bracket attributes, preserved in the order they were applied. */
    readonly attrs: readonly Attribute[];
    /** Fixed dimension of `vector<N>`. */
    readonly dim?: number;
    /** Wrapped descriptor of `optional` and `list`. */
    readonly inner?: AnyTypeDescriptor;
    /** Target declaration of `&T`. */
    readonly target?: ReferenceTarget;
    /** Phantom author-side value type. Never present at runtime. */
    readonly [PHANTOM]: V;
    /** Wrap as `T?`. */
    optional(): TypeDescriptor<"optional", V | null>;
    /** Wrap as `[T]`. */
    list(): TypeDescriptor<"list", V[]>;
    /** Append the `[primary]` attribute. */
    primary(): TypeDescriptor<K, V>;
    /** Append the `[unique]` attribute. */
    unique(): TypeDescriptor<K, V>;
    /** Append an arbitrary bracket attribute, preserving it losslessly. */
    attribute(name: string, ...args: readonly (string | number)[]): TypeDescriptor<K, V>;
    /** Validate a runtime value; throws VosValidationError on failure. */
    parse(value: unknown): V;
    /** Validate a runtime value without throwing. */
    safeParse(value: unknown): SafeParseResult<V>;
}

/** A named field: a scalar descriptor plus its VOS field name. */
export interface FieldDescriptor<Name extends string = string, V = unknown> extends TypeDescriptor<ScalarKind, V> {
    readonly name: Name;
}

/** A `&T` reference descriptor. */
export interface RefDescriptor<Target extends string = string> extends TypeDescriptor<"ref", ReferenceKey> {
    readonly target: ReferenceTarget & { readonly name: Target };
}

/** Any scalar descriptor, used where the concrete kind is not yet known. */
export type AnyTypeDescriptor = TypeDescriptor<ScalarKind, unknown>;

/** Author-side value type of a descriptor. */
export type Infer<D> = D extends { readonly [PHANTOM]: infer V } ? V : never;

interface DescriptorOptions {
    readonly name?: string;
    readonly attrs?: readonly Attribute[];
    readonly dim?: number;
    readonly inner?: AnyTypeDescriptor;
    readonly target?: ReferenceTarget;
}

const DESCRIPTOR_PROTO = {
    optional(this: AnyTypeDescriptor) {
        return makeDescriptor("optional", { inner: this });
    },
    list(this: AnyTypeDescriptor) {
        return makeDescriptor("list", { inner: this });
    },
    primary(this: AnyTypeDescriptor) {
        return withAttribute(this, "primary");
    },
    unique(this: AnyTypeDescriptor) {
        return withAttribute(this, "unique");
    },
    attribute(this: AnyTypeDescriptor, name: string, ...args: readonly (string | number)[]) {
        return withAttribute(this, name, ...args.map(String));
    },
    /** Validate a runtime value against this scalar descriptor; throws on failure. */
    parse<V>(this: TypeDescriptor<ScalarKind, V>, value: unknown): V {
        return validateParse(this as AnyTypeDescriptor, value) as V;
    },
    /** Validate a runtime value against this scalar descriptor without throwing. */
    safeParse<V>(this: TypeDescriptor<ScalarKind, V>, value: unknown): SafeParseResult<V> {
        return validateSafeParse<V>(this as AnyTypeDescriptor, value);
    },
};

/**
 * Build a descriptor as plain data on a shared prototype. Serialization and
 * cloning therefore see data only, never builder functions.
 */
export function makeDescriptor<K extends ScalarKind, V>(kind: K, options: DescriptorOptions = {}): TypeDescriptor<K, V> {
    const descriptor = Object.create(DESCRIPTOR_PROTO) as Record<string, unknown>;
    if (options.name !== undefined) {
        descriptor.name = options.name;
    }
    descriptor.kind = kind;
    descriptor.attrs = options.attrs ?? [];
    if (options.dim !== undefined) {
        descriptor.dim = options.dim;
    }
    if (options.inner !== undefined) {
        descriptor.inner = options.inner;
    }
    if (options.target !== undefined) {
        descriptor.target = options.target;
    }
    return descriptor as unknown as TypeDescriptor<K, V>;
}

/** Name a field descriptor without mutating the source descriptor. */
export function withName<K extends ScalarKind, V, N extends string>(descriptor: TypeDescriptor<K, V>, name: N): FieldDescriptor<N, V> {
    return makeDescriptor<K, V>(descriptor.kind, {
        name,
        attrs: descriptor.attrs,
        dim: descriptor.dim,
        inner: descriptor.inner,
        target: descriptor.target,
    }) as unknown as FieldDescriptor<N, V>;
}

function withAttribute<K extends ScalarKind, V>(
    descriptor: TypeDescriptor<K, V>,
    name: string,
    ...args: readonly string[]
): TypeDescriptor<K, V> {
    const source = descriptor as TypeDescriptor<K, V> & { readonly name?: string };
    return makeDescriptor<K, V>(descriptor.kind, {
        name: source.name,
        attrs: [...descriptor.attrs, { name, args }],
        dim: descriptor.dim,
        inner: descriptor.inner,
        target: descriptor.target,
    });
}
