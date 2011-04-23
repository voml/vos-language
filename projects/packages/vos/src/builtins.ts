/**
 * The `v` type builder: a 1:1 projection of the frozen VOS type algebra.
 *
 * Every builder name maps to a VOS type listed in `vos/documentation/builtins.md`.
 * There is deliberately no `string()` / `number()` / `json()` / `any()`: merging
 * `utf8` with `utf16` or collapsing `i64` into a JS `number` would lose wire,
 * layout and validation semantics.
 */

import { type AnyTypeDescriptor, type Infer, makeDescriptor, type TypeDescriptor } from "./descriptor.js";
import type { VosFileRef } from "./values.js";

/**
 * VOS type builders.
 *
 * ```ts
 * const User = v.table("User", {
 *     user_id: v.uuid().primary(),
 *     user_name: v.utf8().unique(),
 *     avatar: v.file().optional(),
 * });
 * ```
 */
export const v = {
    i8: () => makeDescriptor<"i8", number>("i8"),
    i16: () => makeDescriptor<"i16", number>("i16"),
    i32: () => makeDescriptor<"i32", number>("i32"),
    i64: () => makeDescriptor<"i64", bigint>("i64"),
    u8: () => makeDescriptor<"u8", number>("u8"),
    u16: () => makeDescriptor<"u16", number>("u16"),
    u32: () => makeDescriptor<"u32", number>("u32"),
    u64: () => makeDescriptor<"u64", bigint>("u64"),
    f32: () => makeDescriptor<"f32", number>("f32"),
    f64: () => makeDescriptor<"f64", number>("f64"),
    bool: () => makeDescriptor<"bool", boolean>("bool"),
    utf8: () => makeDescriptor<"utf8", string>("utf8"),
    utf16: () => makeDescriptor<"utf16", string>("utf16"),
    uuid: () => makeDescriptor<"uuid", string>("uuid"),
    decimal: () => makeDescriptor<"decimal", string>("decimal"),
    date: () => makeDescriptor<"date", string>("date"),
    time: () => makeDescriptor<"time", string>("time"),
    datetimeUtc: () => makeDescriptor<"datetimeUtc", string>("datetimeUtc"),
    bytes: () => makeDescriptor<"bytes", Uint8Array>("bytes"),
    file: () => makeDescriptor<"file", VosFileRef>("file"),

    /** Fixed-dimension embedding `vector<N>`. */
    vector: (dim: number): TypeDescriptor<"vector", Float32Array> => {
        if (!Number.isInteger(dim) || dim < 1) {
            throw new RangeError(`v.vector(dim): dim must be a positive integer, got ${dim}`);
        }
        return makeDescriptor<"vector", Float32Array>("vector", { dim });
    },

    /** Optional wrapper `T?`; the author-side value becomes nullable. */
    optional: <T extends AnyTypeDescriptor>(inner: T): TypeDescriptor<"optional", Infer<T> | null> =>
        makeDescriptor<"optional", Infer<T> | null>("optional", { inner }),

    /** List wrapper `[T]`. */
    list: <T extends AnyTypeDescriptor>(inner: T): TypeDescriptor<"list", Infer<T>[]> => makeDescriptor<"list", Infer<T>[]>("list", { inner }),
};

/** Builtin builder names, useful for conformance checks and tooling. */
export type VBuiltinName = Exclude<keyof typeof v, "vector" | "optional" | "list">;
