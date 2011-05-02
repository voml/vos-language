//! WebAssembly export surface for the VOS language toolchain.
//!
//! The `@voml/vos-language` npm package ships the wasm-bindgen output built from
//! this crate under its `lib/` directory, and the TypeScript wrapper in `src/`
//! loads it. The surface is deliberately a narrow JSON ABI so the host depends
//! on no `js_sys` object shape.
//!
//! Syntax ownership stays with `oak-vos`; this crate only forwards the `vos`
//! facade result and never owns a parser, a CST or an AST of its own.

#![warn(missing_docs)]

use wasm_bindgen::prelude::*;

/// Checks VOS source and returns a JSON envelope.
///
/// Shape: `{"ok":true,"diagnostics":[]}` on success, and
/// `{"ok":false,"diagnostics":[{"message":"..."}]}` on failure.
#[wasm_bindgen]
pub fn check(source: &str) -> String {
    match vos::validate_schema(source) {
        Ok(_) => envelope(true, Vec::new()),
        Err(message) => envelope(false, vec![message]),
    }
}

fn envelope(ok: bool, messages: Vec<String>) -> String {
    let diagnostics = messages
        .into_iter()
        .map(|message| serde_json::json!({ "message": message }))
        .collect::<Vec<_>>();
    serde_json::json!({ "ok": ok, "diagnostics": diagnostics }).to_string()
}
