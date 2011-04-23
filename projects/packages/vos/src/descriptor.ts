/**
 * VOS type descriptor protocol.
 *
 * A descriptor carries plain, serializable data (`kind`, `attrs`, …) plus a few
 * builder helpers. It is **not** a stable database ABI: a descriptor must be
 * compiled into canonical VOS source or a canonical VOS artifact before any
 * host consumes it.
 *
 * The protocol intentionally mirrors the frozen VOS type algebra in
 * `vos/documentation/builtins.md`. Builtin names are case-sensitive and the
 * public surface must not offer a wide `string()` / `number()` / `json()`.
 */

declare const PHANTOM: unique symbol;

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
export type WrapperKind = "optional" | "list";

/** Every scalar descriptor kind: builtin or wrapper. */
export type ScalarKind = BuiltinKind | WrapperKind;

/** A VOS scalar type descriptor. */
export interface TypeDescriptor<K extends ScalarKind = ScalarKind, V = unknown> {
    readonly kind: K;
    /** Bracket attributes, preserved in the order they were applied. */
    readonly attrs: readonly Attribute[];
    /** Fixed dimension of `vector<N>`. */
    readonly dim?: number;
    /** Wrapped descriptor of `optional` and `list`. */
    readonly inner?: AnyTypeDescriptor;
    /** Phantom author-side value type. Never present at runtime. */
    readonly [PHANTOM]: V;
    /** Append the `[primary]` attribute. */
    primary(): TypeDescriptor<K, V>;
    /** Append the `[unique]` attribute. */
    unique(): TypeDescriptor<K, V>;
    /** Append an arbitrary bracket attribute, preserving it losslessly. */
    attribute(name: string, ...args: readonly (string | number)[]): TypeDescriptor<K, V>;
}

/** Any scalar descriptor, used where the concrete kind is not yet known. */
export type AnyTypeDescriptor = TypeDescriptor<ScalarKind, unknown>;

/** Author-side value type of a descriptor. */
export type Infer<D> = D extends { readonly [PHANTOM]: infer V } ? V : never;

interface DescriptorOptions {
    readonly attrs?: readonly Attribute[];
    readonly dim?: number;
    readonly inner?: AnyTypeDescriptor;
}

const DESCRIPTOR_PROTO = {
    primary(this: AnyTypeDescriptor) {
        return withAttribute(this, "primary");
    },
    unique(this: AnyTypeDescriptor) {
        return withAttribute(this, "unique");
    },
    attribute(this: AnyTypeDescriptor, name: string, ...args: readonly (string | number)[]) {
        return withAttribute(this, name, ...args.map(String));
    },
};

/**
 * Build a descriptor as plain data on a shared prototype. `JSON.stringify` and
 * `structuredClone` therefore see data only, never builder functions.
 */
export function makeDescriptor<K extends ScalarKind, V>(kind: K, options: DescriptorOptions = {}): TypeDescriptor<K, V> {
    const descriptor = Object.create(DESCRIPTOR_PROTO) as Record<string, unknown>;
    descriptor.kind = kind;
    descriptor.attrs = options.attrs ?? [];
    if (options.dim !== undefined) {
        descriptor.dim = options.dim;
    }
    if (options.inner !== undefined) {
        descriptor.inner = options.inner;
    }
    return descriptor as unknown as TypeDescriptor<K, V>;
}

function withAttribute<K extends ScalarKind, V>(
    descriptor: TypeDescriptor<K, V>,
    name: string,
    ...args: readonly string[]
): TypeDescriptor<K, V> {
    return makeDescriptor<K, V>(descriptor.kind, {
        attrs: [...descriptor.attrs, { name, args }],
        dim: descriptor.dim,
        inner: descriptor.inner,
    });
}
