//! VOS builtin `uuid()` — **UUID v7 only** (RFC 9562).
//!
//! ## Why not v4?
//!
//! Using random UUID v4 as a **clustered primary key** (MySQL `BINARY(16)` / InnoDB PK,
//! or any B-tree ordered by key) inserts rows at random positions across the index tree.
//! That causes **frequent page splits**, higher write amplification, buffer-pool churn,
//! and insert throughput collapse at scale.
//!
//! UUID v7 is **time-ordered** in the high bits, so new keys mostly append to the right
//! edge of the index — the same locality property as auto-increment, while staying opaque
//! and distributed-friendly.
//!
//! Iris and hosts must not expose v4 generators for schema `uuid` columns or `uuid()` calls.

use uuid::Uuid;

/// Generate a canonical hyphenated UUID string. **Always version 7.**
pub fn uuid() -> String {
    Uuid::now_v7().to_string()
}

/// Returns true when `text` parses as UUID version 7.
pub fn is_v7(text: &str) -> bool {
    Uuid::parse_str(text)
        .ok()
        .is_some_and(|u| u.get_version_num() == 7)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_builtin_is_v7() {
        assert!(is_v7(&uuid()));
    }
}
