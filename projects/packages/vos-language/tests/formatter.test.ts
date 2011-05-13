import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { beforeAll, describe, expect, it } from "vitest";
import { formatSource } from "../src/formatter.js";
import { initWasm } from "../src/index.js";

// Node 不能对 `file:` URL 用 fetch，所以测试显式把产物字节交给 wasm。
const wasmBytes = readFileSync(fileURLToPath(new URL("../lib/vos_wasm_bg.wasm", import.meta.url)));

beforeAll(async () => {
    await initWasm(wasmBytes);
});

describe("formatSource", () => {
    it("replays the author's spelling byte for byte in preserve mode", async () => {
        const source = "table  User{@@id:uuid,name:utf8}";
        const result = await formatSource(source, "preserve");
        expect(result.ok).toBe(true);
        expect(result.text).toBe(source);
        expect(result.diagnostics).toEqual([]);
    });

    it("rewrites the layout of a decomposed table in canonical mode", async () => {
        const result = await formatSource("table  User{@@id:uuid,name:utf8}", "canonical");
        expect(result.ok).toBe(true);
        expect(result.text).toBe("table User {\n    @@id: uuid,\n    name: utf8,\n}\n");
    });

    it("normalizes static paths to `::` in canonical mode", async () => {
        const result = await formatSource("using  foo.bar  \ntable T { @@id: uuid }", "canonical");
        expect(result.ok).toBe(true);
        expect(result.text).toBe("using foo::bar\n\ntable T {\n    @@id: uuid,\n}\n");
    });

    it("defaults to canonical mode", async () => {
        const result = await formatSource("table User { @@id: uuid }");
        expect(result.text).toBe("table User {\n    @@id: uuid,\n}\n");
    });

    it("reports a diagnostic instead of throwing on an unknown mode", async () => {
        // @ts-expect-error 故意传入不存在的模式，验证边界校验。
        const result = await formatSource("table User { @@id: uuid }", "weird");
        expect(result.ok).toBe(false);
        expect(result.diagnostics[0]?.message.length).toBeGreaterThan(0);
    });

    it("reports a diagnostic instead of throwing on unparsable source", async () => {
        const result = await formatSource("table User { id: ", "canonical");
        expect(result.ok).toBe(false);
        expect(result.text).toBeUndefined();
        expect(result.diagnostics[0]?.message.length).toBeGreaterThan(0);
    });
});
