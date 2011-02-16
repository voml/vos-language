//! Hand-written lexer and parser for **VOS — Virtual Object Schema**.
//!
//! This crate owns grammar and semantic checking for the shared language
//! contract. Storage products (YYDB / YYDS) and generators consume the AST;
//! they must not fork a parallel dialect.
//!
//! Language failures are reported as lightweight [`vos_ast::Diagnostics`] and
//! converted to **miette** reports via [`error::report_diagnostics`] so every
//! error keeps source bytes, span labels, codes, and related diagnostics.
//!
//! - Schema documents: [`parse_document`]
//! - Expression / operation programs: [`parse_program`]

#![warn(missing_docs)]

mod error;
mod program;

pub use error::{RelatedDiagnostic, VosError, report_diagnostic, report_diagnostics};
pub use program::parse_program;

use vos_ast::{
    BuiltinType, Class, Diagnostic, Diagnostics, Document, EnumVariant, Enums, Field,
    FieldAttribute, Flags, Item, Literal, NamespacePath, Obsolete, Span, Table, TypeExpr,
};

/// Normalize source for parsing and conformance (`*.normalized.vos`).
///
/// Strips a leading UTF-8 BOM and maps `\r\n` / `\r` to `\n`.
pub fn normalize_source(source: &str) -> String {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    source.replace("\r\n", "\n").replace('\r', "\n")
}

/// Parse VOS source into a [`Document`] without semantic validation.
pub fn parse(source: &str) -> Result<Document, Diagnostics> {
    let source = normalize_source(source);
    let mut parser = Parser::new(&source);
    match parser.parse_document() {
        Ok(items_ns) => Ok(Document {
            namespace: items_ns.0,
            items: items_ns.1,
            source,
        }),
        Err(diag) => Err(Diagnostics { errors: vec![diag] }),
    }
}

/// Semantic checks on a parsed document (keys, duplicates, references).
pub fn check(document: &Document) -> Result<(), Diagnostics> {
    let mut diags = Diagnostics::default();
    check_document(document, &mut diags);
    if diags.is_empty() { Ok(()) } else { Err(diags) }
}

/// Parse then check. Preferred entry for hosts such as YYDB.
pub fn parse_document(source: &str) -> Result<Document, Diagnostics> {
    let document = parse(source)?;
    check(&document)?;
    Ok(document)
}

fn check_document(document: &Document, diags: &mut Diagnostics) {
    let mut type_names = Vec::new();
    for item in &document.items {
        let (name, span) = match item {
            Item::Table(t) => (t.name.as_str(), t.span),
            Item::Class(c) => (c.name.as_str(), c.span),
            Item::Enums(e) => (e.name.as_str(), e.span),
            Item::Flags(f) => (f.name.as_str(), f.span),
            Item::Obsolete(_) => continue,
            _ => continue,
        };
        if type_names.iter().any(|(n, _)| n == name) {
            diags.push(Diagnostic::new(
                format!("duplicate type name `{name}`"),
                span,
                Some("rename one of the conflicting declarations"),
            ));
        }
        type_names.push((name.to_owned(), span));
    }

    let table_names: Vec<String> = document.tables().map(|t| t.name.clone()).collect();

    for table in document.tables() {
        check_fields(&table.fields, &table.name, "table", table.span, true, diags);
        for field in &table.fields {
            check_type_expr(&field.ty, field.span, diags);
            check_references(&field.ty, field.span, &table_names, diags);
        }
    }

    for class in document.classes() {
        check_fields(
            &class.fields,
            &class.name,
            "class",
            class.span,
            false,
            diags,
        );
        for field in &class.fields {
            check_type_expr(&field.ty, field.span, diags);
            check_references(&field.ty, field.span, &table_names, diags);
        }
    }

    for item in &document.items {
        match item {
            Item::Enums(enums) => {
                check_enum_variants(&enums.variants, &enums.name, "enums", enums.span, diags)
            }
            Item::Flags(flags) => {
                check_enum_variants(&flags.variants, &flags.name, "flags", flags.span, diags)
            }
            _ => {}
        }
    }
}

fn check_fields(
    fields: &[Field],
    owner: &str,
    kind: &str,
    owner_span: Span,
    require_primary: bool,
    diags: &mut Diagnostics,
) {
    let mut field_names = Vec::new();
    let mut primary_count = 0usize;
    for field in fields {
        if field_names.iter().any(|n| n == &field.name) {
            diags.push(Diagnostic::new(
                format!("duplicate field `{}` on {kind} `{owner}`", field.name),
                field.span,
                Some("rename or remove the duplicate field"),
            ));
        }
        field_names.push(field.name.clone());
        if field.is_primary() {
            primary_count += 1;
        }
    }

    if !require_primary {
        return;
    }
    if primary_count == 0 {
        diags.push(Diagnostic::new(
            format!("{kind} `{owner}` requires a primary key"),
            owner_span,
            Some("add `@@field: Type` or `[primary] field: Type`"),
        ));
    } else if primary_count > 1 {
        diags.push(Diagnostic::new(
            format!(
                "{kind} `{owner}` declares {primary_count} primary keys; exactly one is required"
            ),
            owner_span,
            Some("keep a single `[primary]` / `@@` field per table"),
        ));
    }
}

fn check_enum_variants(
    variants: &[EnumVariant],
    owner: &str,
    kind: &str,
    owner_span: Span,
    diags: &mut Diagnostics,
) {
    if variants.is_empty() {
        diags.push(Diagnostic::new(
            format!("{kind} `{owner}` requires at least one variant"),
            owner_span,
            Some("add `Name = 1,` inside the body"),
        ));
        return;
    }
    let mut names = Vec::new();
    for variant in variants {
        if names.iter().any(|n| n == &variant.name) {
            diags.push(Diagnostic::new(
                format!("duplicate variant `{}` on {kind} `{owner}`", variant.name),
                variant.span,
                Some("rename or remove the duplicate variant"),
            ));
        }
        names.push(variant.name.clone());
    }
}

fn check_type_expr(ty: &TypeExpr, span: Span, diags: &mut Diagnostics) {
    match ty {
        TypeExpr::Named(name) if name == "Uuid" => {
            diags.push(Diagnostic::new(
                "unknown type `Uuid`",
                span,
                Some("builtin uuid is lowercase only"),
            ));
        }
        TypeExpr::Optional(inner) | TypeExpr::List(inner) | TypeExpr::Reference(inner) => {
            check_type_expr(inner, span, diags);
        }
        _ => {}
    }
}

fn check_references(ty: &TypeExpr, span: Span, tables: &[String], diags: &mut Diagnostics) {
    match ty {
        TypeExpr::Reference(inner) => {
            match inner.as_ref() {
                TypeExpr::Named(name) => {
                    if !tables.iter().any(|t| t == name) {
                        diags.push(Diagnostic::new(
                        format!("unknown reference target `{name}`"),
                        span,
                        Some("declare the target `table` in this document or import it with `using`"),
                    ));
                    }
                }
                TypeExpr::Optional(opt_inner) => {
                    if let TypeExpr::Named(name) = opt_inner.as_ref() {
                        if !tables.iter().any(|t| t == name) {
                            diags.push(Diagnostic::new(
                            format!("unknown reference target `{name}`"),
                            span,
                            Some("declare the target `table` in this document or import it with `using`"),
                        ));
                        }
                    } else {
                        check_references(opt_inner, span, tables, diags);
                    }
                }
                other => check_references(other, span, tables, diags),
            }
        }
        TypeExpr::Optional(inner) | TypeExpr::List(inner) => {
            check_references(inner, span, tables, diags);
        }
        _ => {}
    }
}

struct Parser<'a> {
    src: &'a str,
    i: usize,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Self { src, i: 0 }
    }

    fn parse_document(&mut self) -> Result<(Option<NamespacePath>, Vec<Item>), Diagnostic> {
        if self.src.contains('\0') {
            let at = self.src.find('\0').unwrap_or(0);
            return Err(Diagnostic::new(
                "VOS document must not contain NUL bytes",
                Span::new(at, at + 1),
                Some("remove NUL bytes from the source"),
            ));
        }

        let mut namespace = None;
        let mut items = Vec::new();
        loop {
            self.skip_ws_and_comments();
            if self.eof() {
                break;
            }
            if self.peek_ident_is("namespace") {
                if namespace.is_some() {
                    return Err(Diagnostic::new(
                        "duplicate `namespace` declaration",
                        Span::empty(self.i),
                        Some("keep a single namespace per file"),
                    ));
                }
                namespace = Some(self.parse_namespace()?);
                continue;
            }
            if self.peek_ident_is("using") {
                // Imports are recognized so files can round-trip; resolution is
                // still single-file for this slice.
                self.parse_using()?;
                continue;
            }
            if self.peek_ident_is("table") {
                items.push(Item::Table(self.parse_table()?));
                continue;
            }
            if self.peek_ident_is("class") {
                items.push(Item::Class(self.parse_class()?));
                continue;
            }
            if self.peek_ident_is("enums") {
                items.push(Item::Enums(self.parse_enums()?));
                continue;
            }
            if self.peek_ident_is("flags") {
                items.push(Item::Flags(self.parse_flags()?));
                continue;
            }
            if self.peek_ident_is("obsolete") {
                items.push(Item::Obsolete(self.parse_obsolete()?));
                continue;
            }
            return Err(Diagnostic::new(
                "expected `namespace`, `using`, `table`, `class`, `enums`, `flags`, or `obsolete`",
                Span::empty(self.i),
                Some("service / union arrive in a later parser slice"),
            ));
        }
        Ok((namespace, items))
    }

    fn parse_namespace(&mut self) -> Result<NamespacePath, Diagnostic> {
        let start = self.i;
        self.expect_ident("namespace")?;
        let mut segments = Vec::new();
        segments.push(self.expect_any_ident()?);
        while self.eat_punct_seq("::") {
            segments.push(self.expect_any_ident()?);
        }
        let _ = self.eat_punct(';');
        Ok(NamespacePath {
            segments,
            span: Span::new(start, self.i),
        })
    }

    fn parse_using(&mut self) -> Result<(), Diagnostic> {
        self.expect_ident("using")?;
        let _ = self.expect_any_ident()?;
        while self.eat_punct_seq("::") {
            let _ = self.expect_any_ident()?;
        }
        let _ = self.eat_punct(';');
        Ok(())
    }

    fn parse_table(&mut self) -> Result<Table, Diagnostic> {
        let start = self.i;
        self.expect_ident("table")?;
        let name = self.expect_any_ident()?;
        let fields = self.parse_field_block()?;
        Ok(Table {
            name,
            fields,
            span: Span::new(start, self.i),
        })
    }

    fn parse_class(&mut self) -> Result<Class, Diagnostic> {
        let start = self.i;
        self.expect_ident("class")?;
        let name = self.expect_any_ident()?;
        let fields = self.parse_field_block()?;
        Ok(Class {
            name,
            fields,
            span: Span::new(start, self.i),
        })
    }

    fn parse_field_block(&mut self) -> Result<Vec<Field>, Diagnostic> {
        self.expect_punct('{')?;
        let mut fields = Vec::new();
        loop {
            self.skip_ws_and_comments();
            if self.eat_punct('}') {
                break;
            }
            fields.push(self.parse_field()?);
            self.skip_ws_and_comments();
            let _ = self.eat_punct(',');
        }
        Ok(fields)
    }

    fn parse_enums(&mut self) -> Result<Enums, Diagnostic> {
        let start = self.i;
        self.expect_ident("enums")?;
        let name = self.expect_any_ident()?;
        let variants = self.parse_enum_body()?;
        Ok(Enums {
            name,
            variants,
            span: Span::new(start, self.i),
        })
    }

    fn parse_flags(&mut self) -> Result<Flags, Diagnostic> {
        let start = self.i;
        self.expect_ident("flags")?;
        let name = self.expect_any_ident()?;
        let variants = self.parse_enum_body()?;
        Ok(Flags {
            name,
            variants,
            span: Span::new(start, self.i),
        })
    }

    fn parse_enum_body(&mut self) -> Result<Vec<EnumVariant>, Diagnostic> {
        self.expect_punct('{')?;
        let mut variants = Vec::new();
        loop {
            self.skip_ws_and_comments();
            if self.eat_punct('}') {
                break;
            }
            let start = self.i;
            let name = self.expect_any_ident()?;
            self.expect_punct('=')?;
            let value = self.parse_enum_number()?;
            variants.push(EnumVariant {
                name,
                value,
                span: Span::new(start, self.i),
            });
            self.skip_ws_and_comments();
            let _ = self.eat_punct(',');
        }
        Ok(variants)
    }

    fn parse_enum_number(&mut self) -> Result<String, Diagnostic> {
        self.skip_ws_and_comments();
        let start = self.i;
        if self.src[self.i..].starts_with("0x") || self.src[self.i..].starts_with("0X") {
            self.i += 2;
            if !matches!(self.peek(), Some(c) if c.is_ascii_hexdigit()) {
                return Err(Diagnostic::new(
                    "expected hex digits after 0x",
                    Span::empty(self.i),
                    None::<String>,
                ));
            }
            while matches!(self.peek(), Some(c) if c.is_ascii_hexdigit()) {
                self.bump();
            }
            return Ok(self.src[start..self.i].to_owned());
        }
        if !matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            return Err(Diagnostic::new(
                "expected numeric enum value",
                Span::empty(self.i),
                Some("use decimal or 0x… hex"),
            ));
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.bump();
        }
        Ok(self.src[start..self.i].to_owned())
    }

    fn parse_obsolete(&mut self) -> Result<Obsolete, Diagnostic> {
        let start = self.i;
        self.expect_ident("obsolete")?;
        if self.peek_ident_is("table") {
            self.expect_ident("table")?;
            let name = self.expect_any_ident()?;
            let _ = self.eat_punct(';');
            return Ok(Obsolete::Table {
                name,
                span: Span::new(start, self.i),
            });
        }
        if self.peek_ident_is("field") {
            self.expect_ident("field")?;
            let parent = self.expect_any_ident()?;
            self.expect_punct('.')?;
            let field = self.expect_any_ident()?;
            let _ = self.eat_punct(';');
            return Ok(Obsolete::Field {
                parent,
                field,
                span: Span::new(start, self.i),
            });
        }
        Err(Diagnostic::new(
            "expected `obsolete table` or `obsolete field`",
            Span::empty(self.i),
            None::<String>,
        ))
    }

    fn parse_field(&mut self) -> Result<Field, Diagnostic> {
        let start = self.i;
        let mut attrs = Vec::new();

        if self.eat_punct('@') {
            let primary = self.eat_punct('@');
            let name = self.expect_any_ident()?;
            self.expect_punct(':')?;
            let ty = self.parse_type()?;
            let default = self.parse_optional_default()?;
            if primary {
                attrs.push(FieldAttribute::Primary);
            } else {
                attrs.push(FieldAttribute::Unique);
            }
            return Ok(Field {
                name,
                ty,
                attrs,
                default,
                span: Span::new(start, self.i),
            });
        }

        if self.peek() == Some('[') {
            attrs = self.parse_attr_list()?;
        }

        let name = self.expect_any_ident()?;
        self.expect_punct(':')?;
        let ty = self.parse_type()?;
        let default = self.parse_optional_default()?;
        Ok(Field {
            name,
            ty,
            attrs,
            default,
            span: Span::new(start, self.i),
        })
    }

    fn parse_attr_list(&mut self) -> Result<Vec<FieldAttribute>, Diagnostic> {
        self.expect_punct('[')?;
        let mut attrs = Vec::new();
        loop {
            self.skip_ws_and_comments();
            if self.eat_punct(']') {
                break;
            }
            let attr_start = self.i;
            let name = self.expect_any_ident()?;
            match name.as_str() {
                "primary" => attrs.push(FieldAttribute::Primary),
                "unique" => attrs.push(FieldAttribute::Unique),
                other => {
                    return Err(Diagnostic::new(
                        format!("unsupported field attribute `{other}`"),
                        Span::new(attr_start, self.i),
                        Some("supported now: primary, unique"),
                    ));
                }
            }
            self.skip_ws_and_comments();
            let _ = self.eat_punct(',');
        }
        Ok(attrs)
    }

    fn parse_optional_default(&mut self) -> Result<Option<Literal>, Diagnostic> {
        self.skip_ws_and_comments();
        if !self.eat_punct('=') {
            return Ok(None);
        }
        Ok(Some(self.parse_literal()?))
    }

    fn parse_literal(&mut self) -> Result<Literal, Diagnostic> {
        self.skip_ws_and_comments();
        let start = self.i;
        if self.eat_ident_value("null") {
            return Ok(Literal::Null);
        }
        if self.eat_ident_value("true") {
            return Ok(Literal::Bool(true));
        }
        if self.eat_ident_value("false") {
            return Ok(Literal::Bool(false));
        }
        if self.peek() == Some('"') {
            return Ok(Literal::String(self.parse_string()?));
        }
        if matches!(self.peek(), Some(c) if c == '-' || c.is_ascii_digit()) {
            return self.parse_number_literal();
        }
        if matches!(self.peek(), Some(c) if c.is_ascii_alphabetic() || c == '_') {
            return Ok(Literal::Ident(self.expect_any_ident()?));
        }
        Err(Diagnostic::new(
            "expected default literal",
            Span::empty(start),
            Some("use null, true, false, a number, a string, or an identifier"),
        ))
    }

    fn parse_number_literal(&mut self) -> Result<Literal, Diagnostic> {
        let start = self.i;
        if self.peek() == Some('-') {
            self.bump();
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.bump();
        }
        let mut is_float = false;
        if self.peek() == Some('.') {
            is_float = true;
            self.bump();
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.bump();
            }
        }
        let text = self.src[start..self.i].to_owned();
        if is_float {
            Ok(Literal::Float(text))
        } else {
            Ok(Literal::Int(text))
        }
    }

    fn parse_string(&mut self) -> Result<String, Diagnostic> {
        self.expect_punct('"')?;
        let mut out = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.bump();
                return Ok(out);
            }
            if ch == '\\' {
                self.bump();
                let Some(esc) = self.bump() else {
                    return Err(Diagnostic::new(
                        "unterminated string escape",
                        Span::empty(self.i),
                        None::<String>,
                    ));
                };
                out.push(match esc {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '"' => '"',
                    '\\' => '\\',
                    other => other,
                });
                continue;
            }
            if ch == '\n' {
                return Err(Diagnostic::new(
                    "unterminated string literal",
                    Span::empty(self.i),
                    None::<String>,
                ));
            }
            out.push(ch);
            self.bump();
        }
        Err(Diagnostic::new(
            "unterminated string literal",
            Span::empty(self.i),
            None::<String>,
        ))
    }

    fn parse_type(&mut self) -> Result<TypeExpr, Diagnostic> {
        let mut ty = self.parse_type_primary()?;
        loop {
            self.skip_ws_and_comments();
            if self.eat_punct('?') {
                ty = TypeExpr::Optional(Box::new(ty));
                continue;
            }
            break;
        }
        Ok(ty)
    }

    fn parse_type_primary(&mut self) -> Result<TypeExpr, Diagnostic> {
        self.skip_ws_and_comments();
        if self.eat_punct('&') {
            let inner = self.parse_type_primary()?;
            return Ok(TypeExpr::Reference(Box::new(inner)));
        }
        if self.eat_punct('[') {
            let inner = self.parse_type()?;
            self.expect_punct(']')?;
            return Ok(TypeExpr::List(Box::new(inner)));
        }

        let start = self.i;
        let name = self.expect_any_ident()?;
        if name == "vector" {
            self.expect_punct('<')?;
            let dim_start = self.i;
            let dim_text = self.expect_digits()?;
            let dim: u32 = dim_text.parse().map_err(|_| {
                Diagnostic::new(
                    "invalid vector dimension",
                    Span::new(dim_start, self.i),
                    Some("use `vector<N>` with a positive integer N"),
                )
            })?;
            self.expect_punct('>')?;
            return Ok(TypeExpr::Vector { dim });
        }
        if name == "file" {
            return Ok(TypeExpr::File);
        }
        if name == "DateTime" {
            self.expect_punct('<')?;
            self.expect_ident("UTC")?;
            self.expect_punct('>')?;
            return Ok(TypeExpr::Builtin(BuiltinType::DateTimeUtc));
        }
        if let Some(builtin) = BuiltinType::parse(&name) {
            return Ok(TypeExpr::Builtin(builtin));
        }
        // Multi-segment type path: a::b::User
        let mut path = name;
        while self.eat_punct_seq("::") {
            path.push_str("::");
            path.push_str(&self.expect_any_ident()?);
        }
        let _ = start;
        Ok(TypeExpr::Named(path))
    }

    fn expect_digits(&mut self) -> Result<String, Diagnostic> {
        self.skip_ws_and_comments();
        let start = self.i;
        if !matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            return Err(Diagnostic::new(
                "expected digits",
                Span::empty(self.i),
                None::<String>,
            ));
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.bump();
        }
        Ok(self.src[start..self.i].to_owned())
    }

    fn eof(&self) -> bool {
        self.i >= self.src.len()
    }

    fn peek(&self) -> Option<char> {
        self.src[self.i..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.i += ch.len_utf8();
        Some(ch)
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            while matches!(self.peek(), Some(c) if c.is_whitespace()) {
                self.bump();
            }
            if self.src[self.i..].starts_with("//") {
                while let Some(c) = self.bump() {
                    if c == '\n' {
                        break;
                    }
                }
                continue;
            }
            if self.src[self.i..].starts_with('#') {
                // Docs use `#` line comments in examples.
                while let Some(c) = self.bump() {
                    if c == '\n' {
                        break;
                    }
                }
                continue;
            }
            break;
        }
    }

    fn eat_punct(&mut self, expected: char) -> bool {
        self.skip_ws_and_comments();
        if self.peek() == Some(expected) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn eat_punct_seq(&mut self, expected: &str) -> bool {
        self.skip_ws_and_comments();
        if self.src[self.i..].starts_with(expected) {
            self.i += expected.len();
            true
        } else {
            false
        }
    }

    fn expect_punct(&mut self, expected: char) -> Result<(), Diagnostic> {
        if self.eat_punct(expected) {
            Ok(())
        } else {
            Err(Diagnostic::new(
                format!("expected `{expected}`"),
                Span::empty(self.i),
                None::<String>,
            ))
        }
    }

    fn peek_ident_is(&mut self, expected: &str) -> bool {
        self.skip_ws_and_comments();
        let rest = &self.src[self.i..];
        if !rest.starts_with(expected) {
            return false;
        }
        let after = rest[expected.len()..].chars().next();
        !matches!(after, Some(c) if c.is_ascii_alphanumeric() || c == '_')
    }

    fn eat_ident_value(&mut self, expected: &str) -> bool {
        if self.peek_ident_is(expected) {
            self.i += expected.len();
            true
        } else {
            false
        }
    }

    fn expect_ident(&mut self, expected: &str) -> Result<(), Diagnostic> {
        let start = self.i;
        let got = self.expect_any_ident()?;
        if got == expected {
            Ok(())
        } else {
            Err(Diagnostic::new(
                format!("expected `{expected}`, found `{got}`"),
                Span::new(start, self.i),
                None::<String>,
            ))
        }
    }

    fn expect_any_ident(&mut self) -> Result<String, Diagnostic> {
        self.skip_ws_and_comments();
        let start = self.i;
        let Some(first) = self.peek() else {
            return Err(Diagnostic::new(
                "expected identifier",
                Span::empty(self.i),
                None::<String>,
            ));
        };
        if !(first.is_ascii_alphabetic() || first == '_') {
            return Err(Diagnostic::new(
                "expected identifier",
                Span::empty(self.i),
                None::<String>,
            ));
        }
        self.bump();
        while matches!(self.peek(), Some(c) if c.is_ascii_alphanumeric() || c == '_') {
            self.bump();
        }
        Ok(self.src[start..self.i].to_owned())
    }
}
