//! Rust artifact helpers (Dejavu-backed).

use crate::{Result, rust_file_header, rust_struct};
use serde_json::json;

/// One field on a generated Rust item.
#[derive(Debug, Clone)]
pub struct RustField {
    /// Field identifier as emitted in Rust source.
    pub name: String,
    /// Rust type text for the field (e.g. `String`, `Uuid`).
    pub ty: String,
}

/// Render a small Rust module: header + one struct.
pub fn module_with_struct(
    target: &str,
    struct_name: &str,
    docs: Option<&str>,
    fields: &[RustField],
) -> Result<String> {
    let header = rust_file_header(target)?;
    let pairs: Vec<(&str, &str)> = fields
        .iter()
        .map(|f| (f.name.as_str(), f.ty.as_str()))
        .collect();
    let body = rust_struct(struct_name, docs, &pairs)?;
    Ok(format!("{header}\n{body}"))
}

/// Escape hatch for JSON contexts already shaped for a template.
pub fn render_json(template: &str, ctx: serde_json::Value) -> Result<String> {
    crate::render(template, &ctx)
}

/// Build a field list JSON array (shared by future emitters).
pub fn fields_to_json(fields: &[RustField]) -> serde_json::Value {
    json!(
        fields
            .iter()
            .map(|f| json!({ "name": f.name, "ty": f.ty }))
            .collect::<Vec<_>>()
    )
}
