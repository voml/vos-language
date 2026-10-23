/**
 * Runtime validation tests — Phase 1 strict shape checks.
 *
 * These tests exercise the dispatch layer directly (safeParse / safeParseItem)
 * before wiring parse/safeParse onto descriptor prototypes.
 */

import { describe, expect, it } from "vitest";
import { v } from "../src/builtins.js";
import {
    parse,
    parseItem,
    safeParse,
    safeParseItem,
    VosValidationError,
} from "../src/validate.js";
import type { ValidationIssue } from "../src/validate.js";

function expectFailure<T>(result: { ok: true; value: T } | { ok: false; issues: ValidationIssue[] }): ValidationIssue[] {
    expect(result.ok).toBe(false);
    return (result as { ok: false; issues: ValidationIssue[] }).issues;
}

// ---------- Builtin scalars ----------

describe("builtin scalar success", () => {
    it("i8..u32 accept JS number (finite, non-NaN)", () => {
        expect(safeParse(v.i8(), 42).ok).toBe(true);
        expect(safeParse(v.u8(), 200).ok).toBe(true);
        expect(safeParse(v.i16(), -1000).ok).toBe(true);
        expect(safeParse(v.i32(), 0).ok).toBe(true);
        expect(safeParse(v.u32(), 4_294_967_295).ok).toBe(true);
        expect(safeParse(v.f32(), 3.14).ok).toBe(true);
        expect(safeParse(v.f64(), -1.5e100).ok).toBe(true);
    });

    it("i64/u64 accept JS bigint only", () => {
        expect(safeParse(v.i64(), 21n)).toMatchObject({ ok: true, value: 21n });
        expect(safeParse(v.u64(), 0n).ok).toBe(true);
    });

    it("bool accepts true/false", () => {
        expect(safeParse(v.bool(), true)).toMatchObject({ ok: true, value: true });
        expect(safeParse(v.bool(), false).ok).toBe(true);
    });

    it("utf8/utf16/uuid/decimal/date/time/datetimeUtc accept string", () => {
        for (const kind of [v.utf8(), v.utf16(), v.uuid(), v.decimal(), v.date(), v.time(), v.datetimeUtc()]) {
            expect(safeParse(kind, "hello world").ok).toBe(true);
        }
    });

    it("bytes accepts Uint8Array", () => {
        expect(safeParse(v.bytes(), new Uint8Array([1, 2, 3]))).toMatchObject({
            ok: true,
        });
    });

    it("file accepts VosFileRef shape", () => {
        const ref = { identity: "abc", manifestGeneration: 3, length: 1024n };
        expect(safeParse(v.file(), ref)).toMatchObject({ ok: true, value: ref });
    });

    it("vector<N> accepts Float32Array of exact length", () => {
        expect(safeParse(v.vector(3), new Float32Array([1, 2, 3]))).toMatchObject({ ok: true });
    });
});

describe("builtin scalar failure", () => {
    it("integer widths reject non-number", () => {
        expectFailure(safeParse(v.i32(), "42"));
        expectFailure(safeParse(v.u32(), null));
        expectFailure(safeParse(v.i32(), true));
    });

    it("integer widths reject NaN / Infinity", () => {
        expectFailure(safeParse(v.f64(), NaN));
        expectFailure(safeParse(v.f32(), Infinity));
    });

    it("i64/u64 reject number (no silent coercion)", () => {
        const issues = expectFailure(safeParse(v.i64(), 42));
        expect(issues[0].code).toBe("invalid_type");
        expect(issues[0].expected).toBe("i64");
        expect(issues[0].received).toBe("number");
    });

    it("bool rejects non-boolean", () => {
        expectFailure(safeParse(v.bool(), 1));
        expectFailure(safeParse(v.bool(), "true"));
    });

    it("utf8 rejects non-string", () => {
        expectFailure(safeParse(v.utf8(), 42));
        expectFailure(safeParse(v.utf8(), null));
    });

    it("bytes rejects non-Uint8Array", () => {
        expectFailure(safeParse(v.bytes(), new Uint16Array([1, 2])));
        expectFailure(safeParse(v.bytes(), [1, 2, 3]));
    });

    it("file rejects wrong shape", () => {
        expectFailure(safeParse(v.file(), { identity: "abc" })); // missing fields
        expectFailure(safeParse(v.file(), 42));
    });

    it("vector<N> rejects wrong length", () => {
        expectFailure(safeParse(v.vector(3), new Float32Array([1, 2])));
        expectFailure(safeParse(v.vector(3), new Float32Array([1, 2, 3, 4])));
    });

    it("vector rejects non-Float32Array", () => {
        expectFailure(safeParse(v.vector(2), new Uint8Array([1, 2])));
        expectFailure(safeParse(v.vector(2), [1, 2]));
    });
});

// ---------- Wrappers ----------

describe("optional wrapper", () => {
    it("accepts null", () => {
        expect(safeParse(v.optional(v.utf8()), null)).toMatchObject({ ok: true, value: null });
    });

    it("forwards to inner when not null", () => {
        expect(safeParse(v.optional(v.utf8()), "hi")).toMatchObject({ ok: true, value: "hi" });
    });

    it("rejects inner invalid value", () => {
        const issues = expectFailure(safeParse(v.optional(v.utf8()), 42));
        expect(issues[0].code).toBe("invalid_type");
    });

    it("rejects undefined (undefined ≠ null)", () => {
        const issues = expectFailure(safeParse(v.optional(v.utf8()), undefined));
        expect(issues[0].code).toBe("invalid_type");
    });
});

describe("list wrapper", () => {
    it("accepts empty list", () => {
        expect(safeParse(v.list(v.i64()), [])).toMatchObject({ ok: true, value: [] });
    });

    it("accepts all valid elements", () => {
        const r = safeParse(v.list(v.i64()), [1n, 2n, 3n]);
        expect(r.ok).toBe(true);
    });

    it("rejects non-array", () => {
        expectFailure(safeParse(v.list(v.i64()), "hi"));
        expectFailure(safeParse(v.list(v.i64()), null));
    });

    it("collects per-element issues with index path", () => {
        const issues = expectFailure(safeParse(v.list(v.utf8()), ["ok", 42, null]));
        expect(issues).toHaveLength(2);
        expect(issues[0].path).toEqual([1]);
        expect(issues[1].path).toEqual([2]);
    });
});

// ---------- parse() throws ----------

describe("parse throws VosValidationError", () => {
    it("success returns value directly", () => {
        expect(parse(v.utf8(), "hello")).toBe("hello");
    });

    it("failure throws with issues", () => {
        try {
            parse(v.utf8(), 42);
            expect.fail("expected throw");
        } catch (e) {
            expect(e).toBeInstanceOf(VosValidationError);
            const err = e as VosValidationError;
            expect(err.issues).toHaveLength(1);
            expect(err.issues[0].path).toEqual([]);
        }
    });
});

// ---------- Item descriptors ----------

describe("table descriptor", () => {
    const User = v.table("User", {
        name: v.utf8().primary(),
        age: v.i64(),
        nickname: v.optional(v.utf8()),
    });

    it("accepts complete valid row", () => {
        const result = safeParseItem(User, { name: "Ada", age: 42n, nickname: "Ada Lovelace" });
        if (!result.ok) throw new Error(`expected ok, got ${JSON.stringify(result.issues)}`);
        const value = result.value as Record<string, unknown>;
        expect(value.age).toBe(42n);
    });

    it("accepts null for optional field", () => {
        const result = safeParseItem(User, { name: "Ada", age: 42n, nickname: null });
        expect(result.ok).toBe(true);
    });

    it("accepts missing optional field", () => {
        const result = safeParseItem(User, { name: "Ada", age: 42n });
        expect(result.ok).toBe(true);
    });

    it("rejects missing required field with field path", () => {
        const issues = expectFailure(safeParseItem(User, { name: "Ada" }));
        expect(issues[0].code).toBe("missing_field");
        expect(issues[0].path).toEqual(["age"]);
    });

    it("rejects non-object input", () => {
        const issues = expectFailure(safeParseItem(User, "just a string"));
        expect(issues[0].code).toBe("invalid_type");
    });

    it("collects per-field issues with field path", () => {
        const issues = expectFailure(safeParseItem(User, { name: 42, age: "string" }));
        expect(issues).toHaveLength(2);
        expect(issues[0].path).toEqual(["name"]);
        expect(issues[1].path).toEqual(["age"]);
    });

    it("accepts list<scalar> field", () => {
        const Tag = v.object({ tags: v.list(v.utf8()) });
        expect(safeParseItem(Tag, { tags: ["a", "b", "c"] }).ok).toBe(true);
    });

    it("collects issues nested through list with field+index path", () => {
        const Tag = v.object({ tags: v.list(v.utf8()) });
        const issues = expectFailure(safeParseItem(Tag, { tags: ["a", 42, null] }));
        expect(issues[0].path).toEqual(["tags", 1]);
        expect(issues[1].path).toEqual(["tags", 2]);
    });
});

describe("class descriptor", () => {
    const Point = v.class("Point", { x: v.f64(), y: v.f64() });

    it("accepts valid row", () => {
        expect(safeParseItem(Point, { x: 1.5, y: 2.5 }).ok).toBe(true);
    });

    it("rejects wrong type", () => {
        expectFailure(safeParseItem(Point, { x: 1.5, y: "not a number" }));
    });
});

describe("object descriptor", () => {
    const Filter = v.object({ search: v.optional(v.utf8()), limit: v.optional(v.i64()) });

    it("accepts nulls for all optionals", () => {
        expect(safeParseItem(Filter, {}).ok).toBe(true);
    });
});

// ---------- ref (Phase 1: no-op accept) ----------

describe("ref descriptor — Phase 1 no-op", () => {
    const Target = v.table("Target", { id: v.uuid().primary() });
    const ref = v.ref(Target);
    it("accepts anything (no catalog validation)", () => {
        expect(safeParse(ref, "some-key").ok).toBe(true);
        expect(safeParse(ref, 42n).ok).toBe(true);
    });
});
