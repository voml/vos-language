//! VOS artifact generation backed by Dejavu.
//!
//! Emitters may produce source, SQL, OpenAPI, docs, or other text artifacts —
//! not only “code”. Default feature `aot` pre-parses `.dejavu` templates to IR
//! at build time. Enable `dyn` (and disable default features) to parse from
//! source on each render. See `docs/generator.md`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use dejavu::{Dejavu, IrDocument};
use once_cell::sync::OnceCell;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Mutex;

mod targets;

/// Generator error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No registered Dejavu template under this stem name.
    #[error("unknown dejavu template `{0}`")]
    UnknownTemplate(String),
    /// Dejavu render / parse failure.
    #[error("{0}")]
    Dejavu(String),
    /// Embedded AOT IR JSON could not be deserialized.
    #[error("invalid AOT IR for `{0}`: {1}")]
    InvalidIr(String, String),
}

/// Result alias.
pub type Result<T> = std::result::Result<T, Error>;

include!(concat!(env!("OUT_DIR"), "/aot_registry.rs"));

#[allow(dead_code)] // used from aot_registry when `aot` is enabled
fn render_aot_ir(ir_json: &'static str, ctx: &Value) -> Result<String> {
    // Cache deserialized IR per template body (static str identity).
    static CACHE: OnceCell<Mutex<HashMap<usize, IrDocument>>> = OnceCell::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let key = ir_json.as_ptr() as usize;
    let doc = {
        let mut guard = cache.lock().expect("ir cache lock");
        if let Some(doc) = guard.get(&key) {
            doc.clone()
        } else {
            let doc: IrDocument = serde_json::from_str(ir_json)
                .map_err(|e| Error::InvalidIr("embedded".into(), e.to_string()))?;
            guard.insert(key, doc.clone());
            doc
        }
    };
    Dejavu::render(&doc, ctx).map_err(|e| Error::Dejavu(e.to_string()))
}

#[allow(dead_code)] // used from aot_registry when `aot` is disabled
fn render_dyn_source(source: &str, ctx: &Value) -> Result<String> {
    Dejavu::render_source(source, ctx).map_err(|e| Error::Dejavu(format!("{e:?}")))
}

/// Render a registered template by stem name (for example `rust_struct`).
///
/// Uses AOT IR when the `aot` feature is enabled (default); otherwise dyn.
pub fn render(name: &str, ctx: &Value) -> Result<String> {
    render_registered(name, ctx)
}

/// Whether this build prefers AOT (parse at build time).
pub const fn prefers_aot() -> bool {
    cfg!(feature = "aot")
}

/// Emit a standard generated-file header for a language target label.
pub fn rust_file_header(target: &str) -> Result<String> {
    render("rust_file_header", &json!({ "target": target }))
}

/// Emit a simple Rust struct from name + fields via Dejavu.
pub fn rust_struct(name: &str, docs: Option<&str>, fields: &[(&str, &str)]) -> Result<String> {
    let fields_json: Vec<Value> = fields
        .iter()
        .map(|(n, ty)| json!({ "name": n, "ty": ty }))
        .collect();
    render(
        "rust_struct",
        &json!({
            "name": name,
            "docs": docs,
            "fields": fields_json,
        }),
    )
}

pub use targets::rust;
