/**
 * `@voml/vos-language/formatter`：VOS 源码格式化。
 *
 * 与检查面同源：实现是 `projects/crates/vos-wasm` 导出的 wasm，TS 侧只做加载与
 * 类型化包装。格式化建在 Oak 无损 CST 上，本包不拥有第二套 lexer / parser。
 */

import { format as wasmFormat } from "../lib/vos_wasm.js";
import { initWasm } from "./wasm.js";
import { readEnvelope, toDiagnostics, type VosDiagnostic } from "./diagnostics.js";

/**
 * 格式化模式，与 Rust 侧 `vos-format::Mode` 一一对应。
 *
 * - `preserve`：逐字节回放作者自己的拼写（同时是 CST 无损性检查）；
 * - `canonical`：重排 Oak 可分解声明的布局，静态路径统一为 `::`，未分解声明原样通过。
 */
export type VosFormatMode = "preserve" | "canonical";

/** `formatSource` 的结果。失败时 `text` 不出现。 */
export type VosFormatResult = {
    ok: boolean;
    text?: string;
    diagnostics: VosDiagnostic[];
};

/**
 * 格式化 VOS 源码。需要 wasm 时按需初始化。
 *
 * 不抛异常：解析与格式化失败都通过 `diagnostics` 表达，调用方按 `ok` 分支。
 */
export async function formatSource(
    source: string,
    mode: VosFormatMode = "canonical",
): Promise<VosFormatResult> {
    await initWasm();
    const envelope = readEnvelope(wasmFormat(source, mode));
    const ok = envelope?.ok === true;
    if (!ok) return { ok: false, diagnostics: toDiagnostics(envelope) };
    return { ok: true, text: typeof envelope?.text === "string" ? envelope.text : "", diagnostics: [] };
}
