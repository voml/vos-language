import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { beforeAll, describe, expect, it } from "vitest";
import { catalogSource, initWasm } from "../src/index.js";

// Node 不能对 `file:` URL 用 fetch，所以测试显式把产物字节交给 wasm。
const wasmBytes = readFileSync(fileURLToPath(new URL("../lib/vos_wasm_bg.wasm", import.meta.url)));

beforeAll(async () => {
    await initWasm(wasmBytes);
});

describe("catalogSource", () => {
    it("projects a table with exactly one primary field", async () => {
        const result = await catalogSource("table User { @@id: uuid, name: utf8 }");
        expect(result.ok).toBe(true);
        expect(result.diagnostics).toEqual([]);

        const types = (result.catalog as { types?: Array<{ canonicalPath?: string[] }> }).types;
        expect(Array.isArray(types)).toBe(true);
        expect(types?.map((item) => item.canonicalPath)).toContainEqual(["User"]);
    });

    it("rejects a table without a primary field", async () => {
        const result = await catalogSource("table User { id: uuid }");
        expect(result.ok).toBe(false);
        expect(result.catalog).toBeUndefined();
        expect(result.diagnostics.length).toBeGreaterThan(0);
    });

    it("reports a diagnostic instead of throwing on unparsable source", async () => {
        const result = await catalogSource("table User { id: ");
        expect(result.ok).toBe(false);
        expect(result.diagnostics[0]?.message.length).toBeGreaterThan(0);
    });
});
