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

export { v, type VBuiltinName, type VItemName, type VWrapperName } from "./builtins.js";
export {
    type AnyTypeDescriptor,
    type Attribute,
    type BuiltinKind,
    type FieldDescriptor,
    type Infer,
    type RefDescriptor,
    type ReferenceTarget,
    type ScalarKind,
    type TypeDescriptor,
    type WrapperKind,
} from "./descriptor.js";
export {
    type ClassDescriptor,
    type FieldAccess,
    type FieldMap,
    type ItemKind,
    type ItemMetadataKey,
    type NamedFieldMap,
    type NamedItemDescriptor,
    type NamedItemKind,
    type ObjectDescriptor,
    type RowOf,
    type StructuralDescriptor,
    type TableDescriptor,
} from "./items.js";
export { emitSource, type EmittableItem } from "./emit.js";
export type { ReferenceKey, VosFileRef } from "./values.js";
export { checkSource, type VosCheckResult, type VosDiagnostic } from "./legacy-check.js";
