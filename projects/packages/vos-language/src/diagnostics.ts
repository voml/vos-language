/**
 * 语言层诊断的公共类型与信封解析，供 `index.ts` 与 `formatter.ts` 复用
 * （内部模块，公开类型由各自入口再导出）。
 */

/** 一条检查诊断。`line` 仅在宿主能定位到行号时出现。 */
export type VosDiagnostic = {
    line?: number;
    message: string;
};

/** Rust 侧信封里的一条诊断，仅用于边界校验。 */
type RawDiagnostic = {
    message?: unknown;
};

/** Rust 侧信封的形状，仅用于边界校验。 */
export type WasmEnvelope = {
    ok?: unknown;
    diagnostics?: unknown;
    text?: unknown;
    catalog?: unknown;
};

/** 读取 wasm 返回值并解析信封。非 JSON 或形状不符时返回 `undefined`。 */
export function readEnvelope(raw: string): WasmEnvelope | undefined {
    try {
        const parsed: unknown = JSON.parse(raw);
        if (parsed !== null && typeof parsed === "object") return parsed as WasmEnvelope;
        return undefined;
    } catch {
        return undefined;
    }
}

/** 把信封里的 `diagnostics` 收敛成结构化诊断。 */
export function toDiagnostics(envelope: WasmEnvelope | undefined): VosDiagnostic[] {
    const raw = envelope !== undefined && Array.isArray(envelope.diagnostics) ? envelope.diagnostics : [];
    return (raw as RawDiagnostic[]).map((item) => ({
        message: String(item?.message ?? ""),
    }));
}
