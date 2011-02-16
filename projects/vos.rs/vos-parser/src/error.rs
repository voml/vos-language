//! miette diagnostics for VOS parse / check failures.
//!
//! `vos-ast::Diagnostic` stays a lightweight syntax payload (code / message /
//! span / hint). This module attaches [`NamedSource`] so every language error
//! can highlight the originating bytes and keep related diagnostics linked.

use std::fmt;

use miette::{Diagnostic, LabeledSpan, NamedSource, SourceSpan};
use thiserror::Error;
use vos_ast::{Diagnostic as AstDiagnostic, Diagnostics, Span};

/// One VOS language failure with source provenance.
#[derive(Debug, Error)]
#[error("{message}")]
pub struct VosError {
    /// Stable code when known.
    pub code: Option<String>,
    /// Human-readable reason (includes code prefix when present).
    pub message: String,
    /// Repair hint.
    pub hint: Option<String>,
    /// Highlighted source.
    pub src: NamedSource<String>,
    /// Primary label span.
    pub span: SourceSpan,
    /// Additional diagnostics from the same batch (related labels).
    pub related: Vec<RelatedDiagnostic>,
}

/// Secondary diagnostic kept as a related label on [`VosError`].
#[derive(Debug, Clone)]
pub struct RelatedDiagnostic {
    /// Stable code when known.
    pub code: Option<String>,
    /// Message.
    pub message: String,
    /// Span in the same source.
    pub span: SourceSpan,
    /// Hint.
    pub hint: Option<String>,
}

impl Diagnostic for VosError {
    fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        self.code
            .as_ref()
            .map(|c| Box::new(c.as_str()) as Box<dyn fmt::Display + 'a>)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        self.hint
            .as_ref()
            .map(|h| Box::new(h.as_str()) as Box<dyn fmt::Display + 'a>)
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        let mut labels = Vec::with_capacity(1 + self.related.len());
        labels.push(LabeledSpan::new(
            Some(self.message.clone()),
            self.span.offset(),
            self.span.len(),
        ));
        for rel in &self.related {
            let text = match &rel.code {
                Some(code) => format!("{code}: {}", rel.message),
                None => rel.message.clone(),
            };
            labels.push(LabeledSpan::new(
                Some(text),
                rel.span.offset(),
                rel.span.len(),
            ));
        }
        Some(Box::new(labels.into_iter()))
    }
}

fn span_to_source_span(span: Span, source_len: usize) -> SourceSpan {
    let start = span.start.min(source_len);
    let end = span.end.min(source_len).max(start);
    let mut len = end.saturating_sub(start);
    if len == 0 {
        // miette requires a non-empty span even at EOF / empty sources.
        len = 1;
    }
    (start, len).into()
}

/// Build a [`miette::Error`] from AST [`Diagnostics`], attaching `source`.
pub fn report_diagnostics(
    source: impl Into<String>,
    name: impl Into<String>,
    diagnostics: Diagnostics,
) -> miette::Error {
    let source = source.into();
    let name = name.into();
    let source_len = source.len();
    let mut errors = diagnostics.errors;
    if errors.is_empty() {
        return VosError {
            code: None,
            message: "VOS validation failed".into(),
            hint: None,
            src: NamedSource::new(name, source),
            span: (0, 1).into(),
            related: Vec::new(),
        }
        .into();
    }

    let primary = errors.remove(0);
    let code = primary.code.clone();
    let message = match &primary.code {
        Some(code) => format!("{code}: {}", primary.message),
        None => primary.message,
    };
    let related = errors
        .into_iter()
        .map(|d| RelatedDiagnostic {
            code: d.code,
            message: d.message,
            span: span_to_source_span(d.span, source_len),
            hint: d.hint,
        })
        .collect();

    VosError {
        code,
        message,
        hint: primary.hint,
        src: NamedSource::new(name, source),
        span: span_to_source_span(primary.span, source_len),
        related,
    }
    .into()
}

/// Convenience: map a single AST diagnostic.
pub fn report_diagnostic(
    source: impl Into<String>,
    name: impl Into<String>,
    diagnostic: AstDiagnostic,
) -> miette::Error {
    report_diagnostics(
        source,
        name,
        Diagnostics {
            errors: vec![diagnostic],
        },
    )
}
