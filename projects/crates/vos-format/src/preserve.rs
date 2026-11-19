//! Preserve mode: re-emit the exact source bytes from the `data.text` CST.

use data_text_vos::{VosRoot, VosSyntaxElement, VosSyntaxNode};

/// Re-emit `source` from its `data.text` CST.
///
/// The result must equal the input byte for byte. A mismatch means `data.text`'s CST
/// dropped or rewrote trivia, which is a losslessness defect in the frontend,
/// not something this crate may compensate for.
pub fn format_preserving(source: &str) -> Result<String, String> {
    let root = data_text_vos::parse(source)?;
    Ok(cst_text(&root))
}

/// Concatenate every token of the CST in source order.
fn cst_text(root: &VosRoot) -> String {
    let mut out = String::new();
    append_node(&root.syntax, &mut out);
    out
}

fn append_node(node: &VosSyntaxNode, out: &mut String) {
    for child in &node.children {
        match child {
            VosSyntaxElement::Node(nested) => append_node(nested, out),
            VosSyntaxElement::Token(token) => out.push_str(&token.text),
        }
    }
}
