import { describe, expect, it } from "vitest";
import { checkSource } from "./legacy-check.js";

describe("checkSource", () => {
    it("accepts a well formed top level item", () => {
        const result = checkSource("table User {\n    @@user_id: uuid,\n}\n");
        expect(result.ok).toBe(true);
        expect(result.diagnostics).toEqual([]);
    });

    it("rejects empty source", () => {
        const result = checkSource("   \n");
        expect(result.ok).toBe(false);
        expect(result.diagnostics[0]?.message).toBe("source is empty");
    });

    it("reports unclosed braces", () => {
        const result = checkSource("table User {\n");
        expect(result.ok).toBe(false);
        expect(result.diagnostics.some((item) => item.message.includes("unclosed '{'"))).toBe(true);
    });

    it("reports a missing top level item", () => {
        const result = checkSource("foo bar");
        expect(result.ok).toBe(false);
        expect(result.diagnostics.some((item) => item.message.startsWith("expected a top-level item"))).toBe(true);
    });

    it("reports positional service parameters", () => {
        const result = checkSource("service S {\n    login(LoginRequest) -> LoginResponse\n}\n");
        expect(result.ok).toBe(false);
        expect(result.diagnostics.some((item) => item.message.includes("requires a named parameter"))).toBe(true);
    });
});
