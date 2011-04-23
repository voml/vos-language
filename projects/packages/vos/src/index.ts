/**
 * Public TypeScript surface for VOS.
 *
 * `import { v } from "@game-gpt/vos"` builds VOS type descriptors. Descriptors
 * are author-side data: they must be compiled into canonical VOS source or a
 * canonical VOS artifact before a host consumes them.
 *
 * The canonical VOS parser, resolver, catalog and semantic diagnostics live in
 * the Oak backed `oak-vos` pipeline exposed by the `vos` Rust facade. This
 * package never parses VOS source itself.
 */

export { v, type VBuiltinName } from "./builtins.js";
export {
    type AnyTypeDescriptor,
    type Attribute,
    type BuiltinKind,
    type Infer,
    type ScalarKind,
    type TypeDescriptor,
    type WrapperKind,
} from "./descriptor.js";
export type { VosFileRef } from "./values.js";
export { checkSource, type VosCheckResult, type VosDiagnostic } from "./legacy-check.js";
