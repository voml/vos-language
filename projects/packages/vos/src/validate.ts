/**
 * Runtime validation for VOS descriptors (Phase 1).
 *
 * This module is deliberately **strict**: it performs shape-only checks
 * (`typeof`, `instanceof`, `Array.isArray`) with no coercion. Phase 1 does
 * not validate numeric ranges (i8 ∈ [-128, 127]) or string formats (uuid
 * regex), and it does not walk cross-`ref` references.
 *
 * Coercion (number↔bigint, JSON↔VOS value) lives in a later codec layer.
 */

import type {
    AnyTypeDescriptor,
    TypeDescriptor,
} from "./descriptor.js";
import type { VosFileRef } from "./values.js";

/** Minimal shape that table / class / object descriptors share. */
interface ItemDescriptor<F extends Record<string, AnyTypeDescriptor> = Record<string, AnyTypeDescriptor>> {
    readonly kind: string;
    readonly fields: F;
}

// ---------- Public types ----------

export interface ValidationIssue {
    /** Stable issue code. */
    code:
        | "invalid_type"
        | "invalid_value"
        | "missing_dim"
        | "dim_mismatch"
        | "missing_field"
        | "missing_ref_target"
        | "ref_not_validated";
    /** Path to the offending value: empty for root, field names or list indices for nested. */
    path: (string | number)[];
    /** Short human-readable message (English). */
    message: string;
    /** Expected VOS kind label, e.g. `"utf8"`, `"list<i64>"`. */
    expected?: string;
    /** Runtime JS type that was actually received, e.g. `"number"`, `"null"`. */
    received?: string;
}

export type SafeParseResult<T> =
    | { ok: true; value: T }
    | { ok: false; issues: ValidationIssue[] };

export class VosValidationError extends Error {
    public readonly issues: readonly ValidationIssue[];
    public constructor(issues: readonly ValidationIssue[]) {
        const first = issues[0]?.message ?? "VOS validation failed";
        super(first);
        this.name = "VosValidationError";
        this.issues = issues;
    }
}

// ---------- Internal dispatch ----------

type Path = (string | number)[];

const JS_TYPE_LABELS: Record<string, string> = {
    bigint: "bigint",
    boolean: "boolean",
    number: "number",
    object: "object",
    string: "string",
    symbol: "symbol",
    undefined: "undefined",
};

function jsTypeLabel(value: unknown): string {
    if (value === null) return "null";
    if (value instanceof Uint8Array) return "Uint8Array";
    if (value instanceof Float32Array) return "Float32Array";
    const t = typeof value;
    return JS_TYPE_LABELS[t] ?? t;
}

function expectedLabel(descriptor: AnyTypeDescriptor): string {
    if (descriptor.kind === "list" && descriptor.inner) {
        return `list<${expectedLabel(descriptor.inner)}>`;
    }
    if (descriptor.kind === "optional" && descriptor.inner) {
        return `optional<${expectedLabel(descriptor.inner)}>`;
    }
    if (descriptor.kind === "vector") {
        const dim = descriptor.dim;
        return dim !== undefined ? `vector<${dim}>` : "vector";
    }
    return descriptor.kind;
}

function issue(
    code: ValidationIssue["code"],
    path: Path,
    message: string,
    extra: Partial<Omit<ValidationIssue, "code" | "path" | "message">> = {},
): ValidationIssue {
    return { code, path, message, ...extra };
}

function badType(descriptor: AnyTypeDescriptor, path: Path, received: unknown): ValidationIssue {
    return issue("invalid_type", path, `Expected ${expectedLabel(descriptor)}, got ${jsTypeLabel(received)}`, {
        expected: expectedLabel(descriptor),
        received: jsTypeLabel(received),
    });
}

// Scalar shape checks: return true when the runtime value has the right JS shape
// for the VOS kind. No coercion, no range check, no format regex.
function checkScalar(descriptor: AnyTypeDescriptor, value: unknown): ValidationIssue[] {
    switch (descriptor.kind) {
        // Integer widths: JS number (no fraction, NaN/Infinity rejected)
        case "i8":
        case "i16":
        case "i32":
        case "u8":
        case "u16":
        case "u32": {
            if (typeof value !== "number" || Number.isNaN(value) || !Number.isFinite(value)) {
                return [badType(descriptor, [], value)];
            }
            return [];
        }
        // 64-bit widths: JS bigint only
        case "i64":
        case "u64": {
            if (typeof value !== "bigint") {
                return [badType(descriptor, [], value)];
            }
            return [];
        }
        // Float widths: JS number
        case "f32":
        case "f64": {
            if (typeof value !== "number" || Number.isNaN(value) || !Number.isFinite(value)) {
                return [badType(descriptor, [], value)];
            }
            return [];
        }
        case "bool": {
            if (typeof value !== "boolean") {
                return [badType(descriptor, [], value)];
            }
            return [];
        }
        case "utf8":
        case "utf16":
        case "uuid":
        case "decimal":
        case "date":
        case "time":
        case "datetimeUtc": {
            if (typeof value !== "string") {
                return [badType(descriptor, [], value)];
            }
            return [];
        }
        case "bytes": {
            if (!(value instanceof Uint8Array)) {
                return [badType(descriptor, [], value)];
            }
            return [];
        }
        case "file": {
            if (typeof value !== "object" || value === null) {
                return [badType(descriptor, [], value)];
            }
            const ref = value as VosFileRef;
            const problems: string[] = [];
            if (typeof ref.identity !== "string") problems.push("identity");
            if (typeof ref.manifestGeneration !== "number") problems.push("manifestGeneration");
            if (typeof ref.length !== "bigint") problems.push("length");
            if (problems.length > 0) {
                return [
                    issue(
                        "invalid_value",
                        [],
                        `Expected VosFileRef with { identity: string, manifestGeneration: number, length: bigint }, missing/wrong: ${problems.join(", ")}`,
                        { expected: "file", received: jsTypeLabel(value) },
                    ),
                ];
            }
            return [];
        }
        case "vector": {
            if (!(value instanceof Float32Array)) {
                return [badType(descriptor, [], value)];
            }
            if (descriptor.dim === undefined) {
                return [
                    issue(
                        "missing_dim",
                        [],
                        "vector descriptor is missing its fixed dimension — cannot validate length",
                        { expected: expectedLabel(descriptor) },
                    ),
                ];
            }
            if (value.length !== descriptor.dim) {
                return [
                    issue(
                        "dim_mismatch",
                        [],
                        `Expected Float32Array of length ${descriptor.dim}, got ${value.length}`,
                        { expected: expectedLabel(descriptor) },
                    ),
                ];
            }
            return [];
        }
        // Wrapper kinds handled by caller — defensive fallback if somehow reaches here
        case "optional":
        case "list":
        case "ref": {
            return [];
        }
        default: {
            return [issue("invalid_type", [], `Unknown descriptor kind: ${(descriptor as TypeDescriptor).kind}`)];
        }
    }
}

// Recursive dispatch: scalar / wrapper / item
function safeParseDescriptor<T>(
    descriptor: AnyTypeDescriptor,
    value: unknown,
    path: Path,
): SafeParseResult<T> {
    // Wrapper kinds first — they need inner dispatch
    if (descriptor.kind === "optional") {
        if (value === null) return { ok: true, value: null as T };
        const inner = descriptor.inner;
        if (!inner) {
            return {
                ok: false,
                issues: [
                    issue(
                        "invalid_value",
                        path,
                        "optional descriptor missing inner type",
                        { expected: "optional" },
                    ),
                ],
            };
        }
        return safeParseDescriptor<T>(inner, value, path);
    }

    if (descriptor.kind === "list") {
        if (!Array.isArray(value)) {
            return { ok: false, issues: [badType(descriptor, path, value)] };
        }
        const inner = descriptor.inner;
        if (!inner) {
            return {
                ok: false,
                issues: [
                    issue("invalid_value", path, "list descriptor missing inner type", { expected: "list" }),
                ],
            };
        }
        const issues: ValidationIssue[] = [];
        const out: unknown[] = [];
        for (let i = 0; i < value.length; i++) {
            const r = safeParseDescriptor(inner, value[i], [...path, i]);
            if (r.ok) {
                out.push(r.value);
            } else {
                issues.push(...r.issues);
            }
        }
        if (issues.length > 0) return { ok: false, issues };
        return { ok: true, value: out as T };
    }

    if (descriptor.kind === "ref") {
        // Phase 1: no cross-reference validation — accept anything, caller should check
        // catalog-level constraints separately.
        return { ok: true, value: value as T };
    }

    // All other kinds are scalars
    const issues = checkScalar(descriptor, value);
    if (issues.length > 0) return { ok: false, issues: issues.map((i) => ({ ...i, path: [...path, ...i.path] })) };
    return { ok: true, value: value as T };
}

// ---------- Public scalar API ----------

/** Parse a scalar descriptor value; throws VosValidationError on failure. */
export function parse<T>(descriptor: AnyTypeDescriptor, value: unknown): T {
    const result = safeParse<T>(descriptor, value);
    if (!result.ok) throw new VosValidationError(result.issues);
    return result.value;
}

/** Parse a scalar descriptor value without throwing. */
export function safeParse<T>(descriptor: AnyTypeDescriptor, value: unknown): SafeParseResult<T> {
    return safeParseDescriptor<T>(descriptor, value, []);
}

// ---------- Item-level (table / class / object) ----------

function safeParseRow<T extends object>(
    item: ItemDescriptor,
    row: unknown,
): SafeParseResult<T> {
    if (typeof row !== "object" || row === null) {
        return {
            ok: false,
            issues: [
                issue("invalid_type", [], `Expected object matching item fields, got ${jsTypeLabel(row)}`, {
                    expected: item.kind,
                    received: jsTypeLabel(row),
                }),
            ],
        };
    }
    const fieldMap = item.fields;
    const issues: ValidationIssue[] = [];
    const out: Record<string, unknown> = {};
    for (const fieldName of Object.keys(fieldMap)) {
        const fieldDesc = fieldMap[fieldName]!;
        const rowObj = row as Record<string, unknown>;
        const hasOwn = Object.prototype.hasOwnProperty.call(rowObj, fieldName);
        // Undefined handling:
        //   - required field + undefined/missing → issue
        //   - optional field + undefined → treat as null
        const isOptional = fieldDesc.kind === "optional";
        let fieldValue: unknown;
        if (hasOwn) {
            fieldValue = rowObj[fieldName];
        } else if (isOptional) {
            fieldValue = null;
        } else {
            issues.push(
                issue("missing_field", [fieldName], `Required field \`${fieldName}\` is missing from input`, {
                    expected: expectedLabel(fieldDesc),
                }),
            );
            continue;
        }

        if (fieldValue === undefined) {
            if (isOptional) {
                fieldValue = null;
            } else {
                issues.push(
                    issue("invalid_type", [fieldName], `Field \`${fieldName}\` is undefined (expected ${expectedLabel(fieldDesc)})`, {
                        expected: expectedLabel(fieldDesc),
                        received: "undefined",
                    }),
                );
                continue;
            }
        }

        const r = safeParseDescriptor(fieldDesc, fieldValue, [fieldName]);
        if (r.ok) {
            out[fieldName] = r.value;
        } else {
            issues.push(...r.issues);
        }
    }
    if (issues.length > 0) return { ok: false, issues };
    return { ok: true, value: out as T };
}

/**
 * Parse a row against an item descriptor (`table` / `class` / `object`).
 * Throws VosValidationError on failure.
 */
export function parseItem<T extends object>(
    item: ItemDescriptor,
    row: unknown,
): T {
    const result = safeParseRow<T>(item, row);
    if (!result.ok) throw new VosValidationError(result.issues);
    return result.value;
}

/** Parse a row against an item descriptor without throwing. */
export function safeParseItem<T extends object>(
    item: ItemDescriptor,
    row: unknown,
): SafeParseResult<T> {
    return safeParseRow<T>(item, row);
}

// ---------- Prototype wiring helper ----------

/**
 * Bind `.parse()` / `.safeParse()` methods to a descriptor object (scalar or item).
 * Used by `DESCRIPTOR_PROTO` and by `makeItem()`-produced descriptors.
 *
 * This is intentionally not re-exported — it is the bridge between the plain-data
 * descriptor shape and the runtime-validation surface.
 */
export function installParseMethods(target: Record<string, unknown>, isItem: boolean): void {
    Object.defineProperty(target, "parse", {
        value: function (this: Record<string, unknown>, value: unknown) {
            if (isItem) {
                return parseItem(this as unknown as ItemDescriptor, value);
            }
            return parse(this as unknown as AnyTypeDescriptor, value);
        },
        enumerable: false,
        writable: false,
        configurable: true,
    });
    Object.defineProperty(target, "safeParse", {
        value: function (this: Record<string, unknown>, value: unknown) {
            if (isItem) {
                return safeParseItem(this as unknown as ItemDescriptor, value);
            }
            return safeParse(this as unknown as AnyTypeDescriptor, value);
        },
        enumerable: false,
        writable: false,
        configurable: true,
    });
}
