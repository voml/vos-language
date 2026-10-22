/**
 * Public TypeScript surface for VOS.
 *
 * `import { v } from "@voml/vos"` builds VOS type descriptors. Descriptors
 * are author-side data: they must be compiled into a canonical VOS artifact
 * before a host consumes them. This package does not render them as VOS source
 * text and never parses VOS source itself — except that `toSource()` below
 * is the author-side bridge that serializes a descriptor tree back into the
 * `.vos` source document the Rust engine's `ensure_schema` expects.
 *
 * The canonical VOS parser, resolver, catalog and semantic diagnostics live in
 * the Oak backed `oak-vos` pipeline exposed by the `vos` Rust facade.
 */

export { v, type VBuiltinName, type VItemName, type VWrapperName } from "./builtins.js";
export { emitSource, toSource, type EmittableItem } from "./serialize.js";
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
export type { ReferenceKey, VosFileRef } from "./values.js";
