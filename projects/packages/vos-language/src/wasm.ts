/**
 * wasm 核心的共享加载点，供 `index.ts` 与 `formatter.ts` 复用（内部模块，不在公开 exports 中）。
 *
 * Rust 侧（`projects/crates/vos-wasm`）是唯一实现，本文件只负责加载。
 */

import initGlue, { type InitInput } from "../lib/vos_wasm.js";

/** wasm 初始化输入：字节、`Response`、`URL` 或已编译的 `WebAssembly.Module`。 */
export type VosWasmInitInput = InitInput;

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
