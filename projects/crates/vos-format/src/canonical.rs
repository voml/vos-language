//! Canonical mode: structural layout for the declarations Oak decomposes.
//!
//! Layout rules are derived from the canonical DDL example in
//! `vos/documentation/ddl.md` and the existing schema fixtures: one declaration
//! per block, four-space field indentation, a single space after the field
//! colon, one trailing comma per field, and the author's attribute spelling
//! kept verbatim.
//!
//! Static paths are normalized to `::` (`paths.md`): `namespace` and `using`
//! are emitted from Oak's resolved path segments, and named types are rendered
//! from `VosTypeSyntax` rather than copied from source trivia.
//!
//! Declarations Oak does not decompose yet are passed through verbatim. Type
//! aliases such as `vec<3>` are kept as written: rewriting them to the builtin
//! spelling is a VOS semantic decision, not a formatting one.

use oak_vos::{VosDeclaration, VosDeclarationKind, VosField, VosTypeArgument, VosTypeSyntax};

/// Indentation unit for block bodies.
const INDENT: &str = "    ";

/// Canonical layout for `source`.
pub fn format_canonical(source: &str) -> Result<String, String> {
    let root = oak_vos::parse(source)?;
    if root.declarations.is_empty() {
        return Ok(source.to_owned());
    }

    let blocks: Vec<String> = root
        .declarations
        .iter()
        .map(|declaration| render_declaration(declaration, source))
        .collect();
    let mut out = blocks.join("\n\n");
    out.push('\n');
    Ok(out)
}

fn render_declaration(declaration: &VosDeclaration, source: &str) -> String {
    match declaration.kind {
        VosDeclarationKind::Namespace => {
            format!("namespace {}", render_path(declaration.path.as_deref()))
        }
        VosDeclarationKind::Using => format!("using {}", render_path(declaration.path.as_deref())),
        VosDeclarationKind::Table => render_fields_block("table", declaration),
        VosDeclarationKind::Class => render_fields_block("class", declaration),
        _ => render_source_slice(declaration, source),
    }
}

fn render_fields_block(keyword: &str, declaration: &VosDeclaration) -> String {
    let name = declaration.name.as_deref().unwrap_or_default();
    let mut out = format!("{keyword} {name} {{\n");
    for field in &declaration.fields {
        out.push_str(&render_field(field));
    }
    out.push('}');
    out
}

fn render_field(field: &VosField) -> String {
    let mut line = String::from(INDENT);
    line.push_str(&render_attributes(field));
    line.push_str(&field.name);
    line.push_str(": ");
    line.push_str(&render_type(&field.type_expr));
    if let Some(default) = &field.default_value {
        line.push_str(" = ");
        line.push_str(default.text.trim());
    }
    line.push_str(",\n");
    line
}

/// Attribute groups are re-emitted from Oak's exact attribute text, so custom
/// arguments and author grouping survive canonical mode.
fn render_attributes(field: &VosField) -> String {
    if field.attributes.is_empty() {
        return String::new();
    }
    let joined = field
        .attributes
        .iter()
        .map(|attribute| attribute.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    if joined.ends_with(']') {
        format!("{joined} ")
    } else {
        joined
    }
}

fn render_type(type_expr: &VosTypeSyntax) -> String {
    match type_expr {
        VosTypeSyntax::Named { path, .. } => render_path(Some(path.as_slice())),
        VosTypeSyntax::Reference { target, .. } => format!("&{}", render_type(target)),
        VosTypeSyntax::Optional { inner, .. } => format!("{}?", render_type(inner)),
        VosTypeSyntax::List { element, .. } => format!("[{}]", render_type(element)),
        VosTypeSyntax::Generic {
            path, arguments, ..
        } => {
            let rendered = arguments
                .iter()
                .map(|argument| match argument {
                    VosTypeArgument::Type(nested) => render_type(nested),
                    VosTypeArgument::Literal(slice) => slice.text.trim().to_owned(),
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", render_path(Some(path.as_slice())), rendered)
        }
    }
}

fn render_path(path: Option<&[String]>) -> String {
    path.unwrap_or_default().join("::")
}

/// Pass an undecomposed declaration through unchanged.
fn render_source_slice(declaration: &VosDeclaration, source: &str) -> String {
    let span = declaration.span;
    source
        .get(span.start..span.end)
        .unwrap_or_default()
        .trim()
        .to_owned()
}
