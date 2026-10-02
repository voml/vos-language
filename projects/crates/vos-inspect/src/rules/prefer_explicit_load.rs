//! `VOS-INSPECT-LOAD-001` — prefer explicit `.load` for association use sites.

use std::collections::BTreeSet;

use vos_ast::expr::{Expr, Stmt};
use vos_ast::op::{QueryPlan, Stage};
use vos_ast::{Document, Program, Span};

use crate::assoc::{collect_assoc_uses, load_selector_field, reference_field_names};
use crate::{InspectConfig, InspectFinding, InspectId, InspectReport};

/// Inspect a lowered [`QueryPlan`].
pub fn check_prefer_explicit_load(
    plan: &QueryPlan,
    document: &Document,
    config: &InspectConfig,
    report: &mut InspectReport,
) {
    let level = config.level(InspectId::PreferExplicitLoad);
    if !level.emits() {
        return;
    }

    let refs = reference_field_names(document, &plan.source.name);
    if refs.is_empty() {
        return;
    }

    let mut explicit = BTreeSet::new();
    for stage in &plan.stages {
        match stage {
            Stage::Load { selector, span } => {
                if let Some(field) = load_selector_field(selector) {
                    explicit.insert(field);
                }
                let _ = span;
            }
            Stage::Filter { predicate, span } => {
                emit_missing(predicate, &refs, &explicit, *span, level, report);
            }
            Stage::Map { projection, span } => {
                emit_missing(projection, &refs, &explicit, *span, level, report);
            }
            Stage::Sort { keys, span } => {
                for key in keys {
                    emit_missing(&key.expr, &refs, &explicit, *span, level, report);
                }
            }
            _ => {}
        }
    }
}

fn emit_missing(
    expr: &Expr,
    refs: &BTreeSet<String>,
    explicit: &BTreeSet<String>,
    span: Span,
    level: crate::InspectLevel,
    report: &mut InspectReport,
) {
    let mut used = BTreeSet::new();
    collect_assoc_uses(expr, None, refs, &mut used);
    for field in used {
        if explicit.contains(&field) {
            continue;
        }
        report.push(InspectFinding::new(
            InspectId::PreferExplicitLoad,
            level,
            format!(
                "{}: association `{field}` is used without an explicit `.load`",
                InspectId::PreferExplicitLoad.code()
            ),
            span,
            Some(format!(
                "association use sites are inferred by default; add `.load(x => x.{field})` if your team wants explicit prefetch, or leave this rule at `allow`"
            )),
        ));
    }
}

/// Walk a [`Program`] expression tree for method pipelines that look like
/// `Table.filter/map/….collect()` and inspect each reconstructed plan fragment.
pub fn check_program(
    program: &Program,
    document: &Document,
    config: &InspectConfig,
    report: &mut InspectReport,
) {
    for stmt in &program.statements {
        if let Stmt::Let(let_) = stmt {
            walk_expr(&let_.value, document, config, report);
        } else if let Stmt::Expr(expr) = stmt {
            walk_expr(expr, document, config, report);
        }
    }
    if let Some(expr) = &program.result {
        walk_expr(expr, document, config, report);
    }
}

fn walk_expr(expr: &Expr, document: &Document, config: &InspectConfig, report: &mut InspectReport) {
    // Prefer lowering a whole method pipeline once (avoids duplicate findings
    // on nested `.map` / `.filter` calls).
    if let Some(plan) = try_lower_pipeline(expr) {
        check_prefer_explicit_load(&plan, document, config, report);
        return;
    }
    match expr {
        Expr::Call { callee, args, .. } => {
            walk_expr(callee, document, config, report);
            for arg in args {
                walk_expr(arg, document, config, report);
            }
        }
        Expr::Member { object, .. }
        | Expr::Unary { expr: object, .. }
        | Expr::Try { expr: object, .. }
        | Expr::StarProj {
            receiver: object, ..
        } => walk_expr(object, document, config, report),
        Expr::Binary { left, right, .. } => {
            walk_expr(left, document, config, report);
            walk_expr(right, document, config, report);
        }
        Expr::StructProj {
            receiver, items, ..
        } => {
            walk_expr(receiver, document, config, report);
            for item in items {
                if let vos_ast::expr::ProjItem::Field(init) = item
                    && let Some(value) = &init.value
                {
                    walk_expr(value, document, config, report);
                }
            }
        }
        Expr::Lambda(lambda) => walk_expr(&lambda.body, document, config, report),
        Expr::List { items, .. } => {
            for item in items {
                walk_expr(item, document, config, report);
            }
        }
        Expr::TypedObject { fields, .. } | Expr::AnonObject { fields, .. } => {
            for field in fields {
                if let Some(value) = &field.value {
                    walk_expr(value, document, config, report);
                }
            }
        }
        Expr::Literal(_) | Expr::Name { .. } => {}
        _ => {}
    }
}

/// Best-effort lower of `Name.method(…).method(…)` into a [`QueryPlan`].
fn try_lower_pipeline(expr: &Expr) -> Option<QueryPlan> {
    let mut methods: Vec<(&str, &[Expr], Span)> = Vec::new();
    let mut cur = expr;
    loop {
        match cur {
            Expr::Call { callee, args, span } => match callee.as_ref() {
                Expr::Member { object, name, .. } => {
                    methods.push((name.as_str(), args.as_slice(), *span));
                    cur = object.as_ref();
                }
                _ => return None,
            },
            Expr::Member {
                object, name, span, ..
            } if name == "all" => {
                methods.push(("all", &[], *span));
                cur = object.as_ref();
            }
            Expr::Name { name, span } => {
                methods.reverse();
                let mut plan = QueryPlan::all(vos_ast::op::TableRef {
                    name: name.clone(),
                    span: *span,
                });
                for (method, args, span) in methods {
                    match method {
                        "all" => {}
                        "load" if args.len() == 1 => {
                            plan.stages.push(Stage::Load {
                                selector: args[0].clone(),
                                span,
                            });
                        }
                        "filter" if args.len() == 1 => {
                            plan.stages.push(Stage::Filter {
                                predicate: args[0].clone(),
                                span,
                            });
                        }
                        "map" if args.len() == 1 => {
                            plan.stages.push(Stage::Map {
                                projection: args[0].clone(),
                                span,
                            });
                        }
                        "sort_by" | "sort_by_desc" if args.len() == 1 => {
                            plan.stages.push(Stage::Sort {
                                keys: vec![vos_ast::op::SortKey {
                                    expr: args[0].clone(),
                                    dir: if method == "sort_by_desc" {
                                        vos_ast::op::SortDir::Desc
                                    } else {
                                        vos_ast::op::SortDir::Asc
                                    },
                                    span,
                                }],
                                span,
                            });
                        }
                        "skip" if args.len() == 1 => {
                            plan.stages.push(Stage::Skip {
                                count: args[0].clone(),
                                span,
                            });
                        }
                        "take" if args.len() == 1 => {
                            plan.stages.push(Stage::Take {
                                count: args[0].clone(),
                                span,
                            });
                        }
                        "collect" | "first" | "count" | "any" | "get" | "delete" | "update" => {}
                        _ => return None,
                    }
                }
                return Some(plan);
            }
            _ => return None,
        }
    }
}
