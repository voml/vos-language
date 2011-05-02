/**
 * `@voml/vos-language` 的公开 TypeScript 面。
 *
 * 实现是 **Rust 导出的 WebAssembly**：`lib/vos_wasm_bg.wasm` 由
 * `projects/crates/vos-wasm` 经 `scripts/build-wasm.mjs` 生成，`lib/` 是产物目录，
 * 不进版本库。本文件只做加载与类型化包装，**不**在 TypeScript 里重新实现任何
 * VOS 解析或检查逻辑——语法与语义的真相源始终是 Rust 侧（`oak-vos` + `vos`）。
 */

import initGlue, { check as wasmCheck, type InitInput } from "../lib/vos_wasm.js";

/** 一条检查诊断。`line` 仅在宿主能定位到行号时出现。 */
export type VosDiagnostic = {
    line?: number;
    message: string;
};

/** `checkSource` 的结果。 */
export type VosCheckResult = {
    ok: boolean;
    diagnostics: VosDiagnostic[];
};

/** wasm 初始化输入：字节、`Response`、`URL` 或已编译的 `WebAssembly.Module`。 */
export type VosWasmInitInput = InitInput;

/** Rust 侧 `check` 返回的 JSON 信封，仅用于边界校验。 */
type WasmEnvelope = {
    ok?: boolean;
    diagnostics?: { message?: string }[];
};

let initPromise: Promise<void> | undefined;

/**
 * 初始化 wasm 核心。可重复调用，只有第一次真正加载。
 *
 * 省略 `module` 时走 wasm-bindgen 的默认加载路径（按 `lib/vos_wasm_bg.wasm` 的
 * URL 取模块），适用于浏览器与 Worker。Node 不能对 `file:` URL 用 `fetch`，
 * 请显式传入 `readFile` 得到的字节。
 */
export function initWasm(module?: VosWasmInitInput): Promise<void> {
    if (!initPromise) {
        const input = module === undefined ? undefined : { module_or_path: module };
        initPromise = initGlue(input).then(() => undefined);
    }
    return initPromise;
}

/**
 * 检查 VOS 源码，返回结构化结果。需要 wasm 时按需初始化。
 *
 * 不抛异常：解析、投影与语义失败都通过 `diagnostics` 表达，调用方按 `ok` 分支。
 */
export async function checkSource(source: string): Promise<VosCheckResult> {
    await initWasm();
    const envelope = JSON.parse(wasmCheck(source)) as WasmEnvelope;
    const diagnostics = Array.isArray(envelope.diagnostics)
        ? envelope.diagnostics.map((item) => ({ message: String(item?.message ?? "") }))
        : [];
    return { ok: envelope.ok === true, diagnostics };
}
