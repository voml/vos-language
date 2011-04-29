/**
 * `@voml/vos-language` 的公开 TypeScript 面。
 *
 * 这里承载 VOS 的**语言层**能力：`.vos` source 的检查与诊断。声明与类型建模
 * （`v.table` / `v.object` / descriptor / `Infer`）属于 `@voml/vos`，不在这里重复。
 *
 * 目前的 `checkSource` 是迁移期实现：它不是 canonical parser。conformance 级的
 * parse / resolve / catalog / 语义诊断由 Oak backed 的 `oak-vos` pipeline 提供。
 */

export { checkSource, type VosCheckResult, type VosDiagnostic } from "./legacy-check.js";
