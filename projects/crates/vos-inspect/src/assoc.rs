//! Association use-site helpers (shared with host inference semantics).

use std::collections::BTreeSet;

use vos_ast::expr::{Expr, Lambda, ProjItem};
use vos_ast::{Document, TypeExpr};

/// `&T` field names on a table.
pub fn reference_field_names(document: &Document, table: &str) -> BTreeSet<String> {
    let Some(table_def) = document.tables().find(|t| t.name == table) else {
        return BTreeSet::new();
    };
    table_def
        .fields
        .iter()
        .filter(|f| reference_target_table(&f.ty).is_some())
        .map(|f| f.name.clone())
        .collect()
}

/// Target table name inside `&T` / `&T?`.
pub fn reference_target_table(ty: &TypeExpr) -> Option<&str> {
    match ty {
        TypeExpr::Reference(inner) => match inner.as_ref() {
            TypeExpr::Named(name) => Some(name.as_str()),
            _ => None,
        },
        TypeExpr::Optional(inner) => reference_target_table(inner),
        _ => None,
    }
}

/// Collect association **entity** use sites (`x.author.field`, `author.field`,
/// `x.author.{鈥`). Plain `x.author` (PK scalar) does not count.
pub fn collect_assoc_uses(
    expr: &Expr,
    param: Option<&str>,
    refs: &BTreeSet<String>,
    out: &mut BTreeSet<String>,
) {
    match expr {
        Expr::Member { object, .. } => {
            if let Some(field) = association_receiver_field(object, param, refs) {
                out.insert(field);
            }
            collect_assoc_uses(object, param, refs, out);
        }
        Expr::StructProj {
            receiver, items, ..
        } => {
            if let Some(field) = association_receiver_field(receiver, param, refs) {
                out.insert(field);
            }
            collect_assoc_uses(receiver, param, refs, out);
            for item in items {
                if let ProjItem::Field(init) = item
                    && let Some(value) = &init.value
                {
                    collect_assoc_uses(value, param, refs, out);
                }
            }
        }
        Expr::StarProj { receiver, .. } => collect_assoc_uses(receiver, param, refs, out),
        Expr::Lambda(Lambda { params, body, .. }) => {
            let nested = if params.len() == 1 {
                Some(params[0].as_str())
            } else {
                param
            };
            collect_assoc_uses(body, nested, refs, out);
        }
        Expr::Binary { left, right, .. } => {
            collect_assoc_uses(left, param, refs, out);
            collect_assoc_uses(right, param, refs, out);
        }
        Expr::Unary { expr, .. } | Expr::Try { expr, .. } => {
            collect_assoc_uses(expr, param, refs, out);
        }
        Expr::Call { callee, args, .. } => {
            collect_assoc_uses(callee, param, refs, out);
            for arg in args {
                collect_assoc_uses(arg, param, refs, out);
            }
        }
        Expr::List { items, .. } => {
            for item in items {
                collect_assoc_uses(item, param, refs, out);
            }
        }
        Expr::TypedObject { fields, .. } | Expr::AnonObject { fields, .. } => {
            for field in fields {
                if let Some(value) = &field.value {
                    collect_assoc_uses(value, param, refs, out);
                }
            }
        }
        Expr::Literal(_) | Expr::Name { .. } => {}
        _ => {}
    }
}

fn association_receiver_field(
    expr: &Expr,
    param: Option<&str>,
    refs: &BTreeSet<String>,
) -> Option<String> {
    match expr {
        Expr::Member { object, name, .. } if refs.contains(name) => match object.as_ref() {
            Expr::Name { name: recv, .. } => {
                if param.map(|p| p == recv).unwrap_or(true) {
                    Some(name.clone())
                } else {
                    None
                }
            }
            _ => None,
        },
        Expr::Name { name, .. } if refs.contains(name) => Some(name.clone()),
        _ => None,
    }
}

/// `.load(x => x.author)` / `.load(x => author)` / `.load("author")` selector.
pub fn load_selector_field(selector: &Expr) -> Option<String> {
    let body = match selector {
        Expr::Lambda(Lambda { params, body, .. }) if params.len() == 1 => body.as_ref(),
        other => other,
    };
    match body {
        Expr::Member { object, name, .. } => match object.as_ref() {
            Expr::Name { .. } => Some(name.clone()),
            _ => None,
        },
        Expr::Name { name, .. } => Some(name.clone()),
        Expr::Literal(vos_ast::Literal::String(s)) => Some(s.clone()),
        _ => None,
    }
}
