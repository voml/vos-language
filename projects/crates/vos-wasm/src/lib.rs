//! WebAssembly export surface for the VOS language toolchain.
//!
//! The `@voml/vos-language` npm package ships the wasm-bindgen output built from
//! this crate under its `lib/` directory, and the TypeScript wrapper in `src/`
//! loads it. The surface is deliberately a narrow JSON ABI so the host depends
//! on no `js_sys` object shape.
//!
//! Syntax ownership stays with `oak-vos`; this crate only forwards the `vos`
//! facade result and never owns a parser, a CST or an AST of its own.

#![deny(missing_docs)]

use wasm_bindgen::prelude::*;

/// Checks VOS source and returns a JSON envelope.
///
/// Shape: `{"ok":true,"diagnostics":[]}` on success, and
/// `{"ok":false,"diagnostics":[{"message":"..."}]}` on failure.
#[wasm_bindgen]
pub fn check(source: &str) -> String {
    match vos::validate_schema(source) {
        Ok(_) => success(None),
        Err(message) => failure(vec![message]),
    }
}

/// Formats VOS source and returns a JSON envelope.
///
/// `mode` selects the spelling contract: `"preserve"` replays the exact source
/// bytes from the Oak lossless CST, and `"canonical"` rewrites the layout of the
/// declarations Oak decomposes structurally. Shape: `{"ok":true,"text":"..."}`
/// on success, and `{"ok":false,"diagnostics":[{"message":"..."}]}` on failure.
#[wasm_bindgen]
pub fn format(source: &str, mode: &str) -> String {
    let mode = match mode {
        "preserve" => vos::format::Mode::Preserve,
        "canonical" => vos::format::Mode::Canonical,
        other => return failure(vec![format!("unknown VOS format mode `{other}`")]),
    };
    match vos::format::format(source, mode) {
        Ok(text) => success(Some(&text)),
        Err(message) => failure(vec![message]),
    }
}

/// Projects the schema catalog of VOS source and returns a JSON envelope.
///
/// The catalog is the resolved `SchemaProjection` artifact, so a host reads the
/// canonical paths, type kinds, fields and attributes without owning a second
/// parser. Shape: `{"ok":true,"catalog":{...}}` on success, and
/// `{"ok":false,"diagnostics":[{"message":"..."}]}` on failure.
#[wasm_bindgen]
pub fn catalog(source: &str) -> String {
    match vos::validate_schema(source) {
        Ok(projection) => match serde_json::to_value(&projection) {
            Ok(catalog) => serde_json::json!({ "ok": true, "catalog": catalog }).to_string(),
            Err(error) => failure(vec![format!("VOS catalog serialization failed: {error}")]),
        },
        Err(message) => failure(vec![message]),
    }
}

/// Envelope for a successful call. `text` is present only for results that
/// carry produced source.
fn success(text: Option<&str>) -> String {
    let mut body = serde_json::json!({ "ok": true, "diagnostics": [] });
    if let Some(text) = text {
        body["text"] = serde_json::Value::String(text.to_owned());
    }
    body.to_string()
}

/// Envelope for a failed call, carrying one diagnostic per message.
fn failure(messages: Vec<String>) -> String {
    let diagnostics = messages
        .into_iter()
        .map(|message| serde_json::json!({ "message": message }))
        .collect::<Vec<_>>();
    serde_json::json!({ "ok": false, "diagnostics": diagnostics }).to_string()
}
