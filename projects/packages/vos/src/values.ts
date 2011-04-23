/**
 * Author-side value carriers for VOS types whose JavaScript built-in type would
 * be lossy.
 *
 * The mapping is frozen in the TypeScript facade design: `i64` / `u64` become
 * `bigint`, `decimal` becomes a decimal string, `date` / `time` /
 * `DateTime<UTC>` become ISO-8601 strings, `bytes` becomes `Uint8Array`,
 * `vector<N>` becomes `Float32Array`, and `file` becomes {@link VosFileRef}.
 */

/** Logical large-file handle: object identity, chunk manifest generation and length. */
export interface VosFileRef {
    /** Object identity of the file manifest. */
    readonly identity: string;
    /** Chunk manifest generation of the published object. */
    readonly manifestGeneration: number;
    /** Total byte length of the file. */
    readonly length: bigint;
}
