import { describe, expect, expectTypeOf, it } from "vitest";
import { v, type VBuiltinName } from "./builtins.js";
import type { Infer } from "./descriptor.js";
import type { VosFileRef } from "./values.js";

/** The frozen VOS builtin list from `vos/documentation/builtins.md`. */
const BUILTIN_NAMES = [
    "i8",
    "i16",
    "i32",
    "i64",
    "u8",
    "u16",
    "u32",
    "u64",
    "f32",
    "f64",
    "bool",
    "utf8",
    "utf16",
    "uuid",
    "decimal",
    "date",
    "time",
    "datetimeUtc",
    "bytes",
    "file",
] as const;

const WRAPPER_NAMES = ["vector", "optional", "list"] as const;

const ITEM_NAMES = ["table", "class", "object", "ref"] as const;

describe("v builtin builders", () => {
    it("exposes exactly the frozen VOS builtin list plus wrappers and item builders", () => {
        expect(Object.keys(v).sort()).toEqual([...BUILTIN_NAMES, ...WRAPPER_NAMES, ...ITEM_NAMES].sort());
    });

    it("never offers a wide string / number / json builder", () => {
        const names: readonly string[] = Object.keys(v);
        for (const forbidden of ["string", "number", "json", "any", "int", "float", "double", "unknown"]) {
            expect(names).not.toContain(forbidden);
        }
    });

    it("keeps every builder name identical to its VOS kind", () => {
        for (const name of BUILTIN_NAMES) {
            const descriptor = v[name]();
            expect(descriptor.kind).toBe(name);
            expect(descriptor.attrs).toEqual([]);
        }
    });

    it("treats the builtin builder names as a closed set", () => {
        expectTypeOf<VBuiltinName>().toEqualTypeOf<(typeof BUILTIN_NAMES)[number]>();
    });
});

describe("v value mapping", () => {
    it("maps 64-bit integers to bigint and 32-bit-or-smaller to number", () => {
        expectTypeOf<Infer<ReturnType<typeof v.i64>>>().toEqualTypeOf<bigint>();
        expectTypeOf<Infer<ReturnType<typeof v.u64>>>().toEqualTypeOf<bigint>();
        expectTypeOf<Infer<ReturnType<typeof v.i32>>>().toEqualTypeOf<number>();
        expectTypeOf<Infer<ReturnType<typeof v.u32>>>().toEqualTypeOf<number>();
        expectTypeOf<Infer<ReturnType<typeof v.f64>>>().toEqualTypeOf<number>();
    });

    it("keeps utf8 and utf16 distinguishable from uuid and decimal", () => {
        expectTypeOf<Infer<ReturnType<typeof v.utf8>>>().toEqualTypeOf<string>();
        expectTypeOf<Infer<ReturnType<typeof v.utf16>>>().toEqualTypeOf<string>();
        expectTypeOf<Infer<ReturnType<typeof v.uuid>>>().toEqualTypeOf<string>();
        expectTypeOf<Infer<ReturnType<typeof v.decimal>>>().toEqualTypeOf<string>();
    });

    it("carries bytes, vector and file as their dedicated runtime types", () => {
        expectTypeOf<Infer<ReturnType<typeof v.bytes>>>().toEqualTypeOf<Uint8Array>();
        expectTypeOf<Infer<ReturnType<typeof v.vector>>>().toEqualTypeOf<Float32Array>();
        expectTypeOf<Infer<ReturnType<typeof v.file>>>().toEqualTypeOf<VosFileRef>();
    });

    it("carries calendar and instant types as ISO-8601 strings", () => {
        expectTypeOf<Infer<ReturnType<typeof v.date>>>().toEqualTypeOf<string>();
        expectTypeOf<Infer<ReturnType<typeof v.time>>>().toEqualTypeOf<string>();
        expectTypeOf<Infer<ReturnType<typeof v.datetimeUtc>>>().toEqualTypeOf<string>();
    });
});

describe("v.vector dimension", () => {
    it("records a positive integer dimension", () => {
        const descriptor = v.vector(3);
        expect(descriptor.kind).toBe("vector");
        expect(descriptor.dim).toBe(3);
    });

    it("rejects non-positive and non-integer dimensions", () => {
        expect(() => v.vector(0)).toThrow(RangeError);
        expect(() => v.vector(-1)).toThrow(RangeError);
        expect(() => v.vector(1.5)).toThrow(RangeError);
        expect(() => v.vector(Number.NaN)).toThrow(RangeError);
    });
});
