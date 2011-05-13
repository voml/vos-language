/**
 * `@voml/vos-language` 的公开 TypeScript 面：检查、catalog 投影与加载入口。
 *
 * 实现是 **Rust 导出的 WebAssembly**：`lib/vos_wasm_bg.wasm` 由
 * `projects/crates/vos-wasm` 经 `scripts/build-wasm.mjs` 生成，`lib/` 是产物目录，
 * 不进版本库。本文件只做加载与类型化包装，**不**在 TypeScript 里重新实现任何
 * VOS 解析或检查逻辑——语法与语义的真相源始终是 Rust 侧（`oak-vos` + `vos`）。
 *
 * 格式化面在 `@voml/vos-language/formatter`，见 `./formatter.js`。
 */

import { check as wasmCheck, catalog as wasmCatalog } from "../lib/vos_wasm.js";
import { initWasm } from "./wasm.js";
import { readEnvelope, toDiagnostics, type VosDiagnostic } from "./diagnostics.js";

export { initWasm, type VosWasmInitInput } from "./wasm.js";
export type { VosDiagnostic } from "./diagnostics.js";

/** `checkSource` 的结果。 */
export type VosCheckResult = {
    ok: boolean;
    diagnostics: VosDiagnostic[];
};

/**
 * `catalogSource` 的结果。
 *
 * `catalog` 是 `vos-contract` 的 `SchemaProjection` 工件，字段名与形状由 Rust 侧的
 * 序列化决定；canonical artifact 合同在 TS 里冻结之前以 `unknown` 交出，调用方需自行
 * 收窄，本包不伪造一层可能漂移的 TS 形状。
 */
export type VosCatalogResult = {
    ok: boolean;
    catalog?: unknown;
    diagnostics: VosDiagnostic[];
};

/**
 * 检查 VOS 源码，返回结构化结果。需要 wasm 时按需初始化。
 *
 * 不抛异常：解析、投影与语义失败都通过 `diagnostics` 表达，调用方按 `ok` 分支。
 */
export async function checkSource(source: string): Promise<VosCheckResult> {
    await initWasm();
    const envelope = readEnvelope(wasmCheck(source));
    return { ok: envelope?.ok === true, diagnostics: toDiagnostics(envelope) };
}

/**
 * 把 VOS 源码投影成 schema catalog（canonical 路径、类型、字段与 attribute）。
 *
 * 不抛异常：解析、投影与语义失败都通过 `diagnostics` 表达；序列化失败同样落为诊断。
 */
export async function catalogSource(source: string): Promise<VosCatalogResult> {
    await initWasm();
    const envelope = readEnvelope(wasmCatalog(source));
    const ok = envelope?.ok === true;
    if (!ok) return { ok: false, diagnostics: toDiagnostics(envelope) };
    return { ok: true, catalog: envelope?.catalog, diagnostics: [] };
}
