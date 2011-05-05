import { describe, expect, expectTypeOf, it } from "vitest";
import { v } from "../src/builtins.js";
import type { Infer } from "../src/descriptor.js";

describe("descriptor attributes", () => {
    it("appends primary and unique without mutating the source descriptor", () => {
        const base = v.uuid();
        const id = base.primary();
        const handle = v.utf8().unique();

        expect(base.attrs).toEqual([]);
        expect(id.attrs).toEqual([{ name: "primary", args: [] }]);
        expect(handle.attrs).toEqual([{ name: "unique", args: [] }]);
    });

    it("preserves open attributes with their arguments in application order", () => {
        const descriptor = v.utf8().attribute("backfill", "anonymous").primary();
        expect(descriptor.attrs).toEqual([
            { name: "backfill", args: ["anonymous"] },
            { name: "primary", args: [] },
        ]);
    });

    it("stringifies numeric attribute arguments", () => {
        const descriptor = v.utf8().attribute("tag", 1);
        expect(descriptor.attrs).toEqual([{ name: "tag", args: ["1"] }]);
    });

    it("keeps attributes on wrapper descriptors too", () => {
        const descriptor = v.list(v.uuid()).primary();
        expect(descriptor.kind).toBe("list");
        expect(descriptor.attrs).toEqual([{ name: "primary", args: [] }]);
    });
});

describe("descriptor is plain data", () => {
    it("serializes to data only, with builder helpers on the prototype", () => {
        const descriptor = v.uuid().primary();
        expect(JSON.parse(JSON.stringify(descriptor))).toEqual({ kind: "uuid", attrs: [{ name: "primary", args: [] }] });
        expect(typeof descriptor.primary).toBe("function");
        expect(Object.keys(descriptor).sort()).toEqual(["attrs", "kind"]);
    });

    it("serializes nested wrappers without functions", () => {
        const descriptor = v.list(v.optional(v.utf8()));
        expect(JSON.parse(JSON.stringify(descriptor))).toEqual({
            kind: "list",
            attrs: [],
            inner: { kind: "optional", attrs: [], inner: { kind: "utf8", attrs: [] } },
        });
    });

    it("carries the vector dimension into serialized data", () => {
        expect(JSON.parse(JSON.stringify(v.vector(768)))).toEqual({ kind: "vector", attrs: [], dim: 768 });
    });
});

describe("wrappers", () => {
    it("keeps the wrapped descriptor by reference", () => {
        const inner = v.utf8();
        expect(v.optional(inner).inner).toBe(inner);
        expect(v.list(inner).inner).toBe(inner);
    });

    it("nests optional and list in either order", () => {
        const optionalOfList = v.optional(v.list(v.uuid()));
        const listOfOptional = v.list(v.optional(v.uuid()));

        expect(optionalOfList.inner?.kind).toBe("list");
        expect(optionalOfList.inner?.inner?.kind).toBe("uuid");
        expect(listOfOptional.inner?.kind).toBe("optional");
        expect(listOfOptional.inner?.inner?.kind).toBe("uuid");
    });

    it("infers nullable and list value types", () => {
        const optionalName = v.optional(v.utf8());
        const names = v.list(v.utf8());
        const optionalIds = v.optional(v.uuid());

        expectTypeOf<Infer<typeof optionalName>>().toEqualTypeOf<string | null>();
        expectTypeOf<Infer<typeof names>>().toEqualTypeOf<string[]>();
        expectTypeOf<Infer<typeof optionalIds>>().not.toEqualTypeOf<string[]>();
    });
});
