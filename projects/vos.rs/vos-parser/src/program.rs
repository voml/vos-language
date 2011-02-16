//! Parser for VOS operation / expression programs (`.vos`).
//!
//! Normative grammar: `docs/expressions.md` + `docs/operations.md`.
use crate::normalize_source;
use vos_ast::codes;
use vos_ast::expr::{
    BinaryOp, Expr, FieldInit, FnDecl, FnKind, FnParam, Lambda, Let, PathSep, Program, ProjItem,
    Stmt, UnaryOp,
};
use vos_ast::{BuiltinType, Diagnostic, Diagnostics, Literal, Span, TypeExpr};

/// Parse a VOS expression program (`let` + optional trailing expression).
pub fn parse_program(source: &str) -> Result<Program, Diagnostics> {
    let source = normalize_source(source);
    let mut parser = ProgramParser::new(&source);
    match parser.parse_program() {
        Ok(program) => Ok(program),
        Err(diag) => Err(Diagnostics { errors: vec![diag] }),
    }
}

struct ProgramParser<'a> {
    src: &'a str,
    i: usize,
}

impl<'a> ProgramParser<'a> {
    fn new(src: &'a str) -> Self {
        Self { src, i: 0 }
    }

    fn parse_program(&mut self) -> Result<Program, Diagnostic> {
        self.parse_program_inner(false)
    }

    /// Parse a program. When `stop_at_rbrace`, stop before a closing `}` (micro/macro body).
    fn parse_program_inner(&mut self, stop_at_rbrace: bool) -> Result<Program, Diagnostic> {
        if self.src.contains('\0') {
            let at = self.src.find('\0').unwrap_or(0);
            return Err(Diagnostic::new(
                "VOS program must not contain NUL bytes",
                Span::new(at, at + 1),
                Some("remove NUL bytes from the source"),
            ));
        }

        let start = self.i;
        let mut micros = Vec::new();
        let mut statements = Vec::new();
        let mut result = None;

        loop {
            self.skip_ws_and_comments();
            if self.eof() {
                break;
            }
            if stop_at_rbrace && self.peek() == Some('}') {
                break;
            }

            if self.peek_ident_is("micro") {
                micros.push(self.parse_fn_decl(FnKind::Micro)?);
                continue;
            }
            if self.peek_ident_is("macro") {
                let at = self.i;
                return Err(Diagnostic::with_code(
                    codes::DDL_SESSION_REQUIRED,
                    "macro declaration changes database DDL",
                    Span::new(at, (at + 5).min(self.src.len())),
                    Some("open a DDL session and apply the macro declaration"),
                ));
            }

            if self.peek_ident_is("let") {
                statements.push(Stmt::Let(self.parse_let()?));
                continue;
            }

            // Trailing expression (program result) or expression statement.
            let expr = self.parse_expression()?;
            self.skip_ws_and_comments();
            if self.eof() || (stop_at_rbrace && self.peek() == Some('}')) {
                result = Some(expr);
                break;
            }
            // More content follows → treat as expression statement.
            statements.push(Stmt::Expr(expr));
        }

        Ok(Program {
            micros,
            statements,
            result,
            span: Span::new(start, self.i),
        })
    }

    fn parse_fn_decl(&mut self, kind: FnKind) -> Result<FnDecl, Diagnostic> {
        let start = self.i;
        let keyword = match kind {
            FnKind::Micro => "micro",
            FnKind::Macro => "macro",
            _ => "micro",
        };
        self.expect_ident(keyword)?;
        let name = self.expect_any_ident()?;
        self.expect_punct('(')?;
        let mut params = Vec::new();
        loop {
            self.skip_ws_and_comments();
            if self.eat_punct(')') {
                break;
            }
            let p_start = self.i;
            let pname = self.expect_any_ident()?;
            self.expect_punct(':')?;
            let ty = self.parse_type_annotation()?;
            params.push(FnParam {
                name: pname,
                ty,
                span: Span::new(p_start, self.i),
            });
            self.skip_ws_and_comments();
            if self.eat_punct(',') {
                continue;
            }
            self.expect_punct(')')?;
            break;
        }
        let return_ty = if self.eat_punct_seq("->") {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };
        self.expect_punct('{')?;
        let body = self.parse_program_inner(true)?;
        self.expect_punct('}')?;
        Ok(FnDecl {
            kind,
            name,
            params,
            return_ty,
            body,
            span: Span::new(start, self.i),
        })
    }

    fn parse_let(&mut self) -> Result<Let, Diagnostic> {
        let start = self.i;
        self.expect_ident("let")?;
        let name = self.expect_any_ident()?;
        let ty = if self.eat_punct(':') {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };
        self.expect_punct('=')?;
        let value = self.parse_expression()?;
        Ok(Let {
            name,
            ty,
            value,
            span: Span::new(start, self.i),
        })
    }

    fn parse_type_annotation(&mut self) -> Result<TypeExpr, Diagnostic> {
        // Minimal annotations: Name, Name?, builtins.
        self.skip_ws_and_comments();
        let name = self.expect_any_ident()?;
        let mut ty = if let Some(builtin) = BuiltinType::parse(&name) {
            TypeExpr::Builtin(builtin)
        } else {
            TypeExpr::Named(name)
        };
        if self.eat_punct('?') {
            ty = TypeExpr::Optional(Box::new(ty));
        }
        Ok(ty)
    }

    fn parse_expression(&mut self) -> Result<Expr, Diagnostic> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, Diagnostic> {
        let mut left = self.parse_and()?;
        loop {
            self.skip_ws_and_comments();
            if self.eat_punct_seq("||") {
                let right = self.parse_and()?;
                let span = Span::new(expr_start(&left), self.i);
                left = Expr::Binary {
                    op: BinaryOp::Or,
                    left: Box::new(left),
                    right: Box::new(right),
                    span,
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, Diagnostic> {
        let mut left = self.parse_cmp()?;
        loop {
            self.skip_ws_and_comments();
            if self.eat_punct_seq("&&") {
                let right = self.parse_cmp()?;
                let span = Span::new(expr_start(&left), self.i);
                left = Expr::Binary {
                    op: BinaryOp::And,
                    left: Box::new(left),
                    right: Box::new(right),
                    span,
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_cmp(&mut self) -> Result<Expr, Diagnostic> {
        let mut left = self.parse_add()?;
        loop {
            self.skip_ws_and_comments();
            let op = if self.eat_punct_seq("==") {
                Some(BinaryOp::Eq)
            } else if self.eat_punct_seq("!=") {
                Some(BinaryOp::Ne)
            } else if self.eat_punct_seq("<=") {
                Some(BinaryOp::Le)
            } else if self.eat_punct_seq(">=") {
                Some(BinaryOp::Ge)
            } else if self.eat_punct('<') {
                Some(BinaryOp::Lt)
            } else if self.eat_punct('>') {
                Some(BinaryOp::Gt)
            } else {
                None
            };
            let Some(op) = op else { break };
            let right = self.parse_add()?;
            let span = Span::new(expr_start(&left), self.i);
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_add(&mut self) -> Result<Expr, Diagnostic> {
        let mut left = self.parse_mul()?;
        loop {
            self.skip_ws_and_comments();
            let op = if self.eat_punct('+') {
                Some(BinaryOp::Add)
            } else if self.eat_punct('-') {
                Some(BinaryOp::Sub)
            } else {
                None
            };
            let Some(op) = op else { break };
            let right = self.parse_mul()?;
            let span = Span::new(expr_start(&left), self.i);
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> Result<Expr, Diagnostic> {
        let mut left = self.parse_unary()?;
        loop {
            self.skip_ws_and_comments();
            let op = if self.eat_punct('*') {
                Some(BinaryOp::Mul)
            } else if self.eat_punct('/') {
                Some(BinaryOp::Div)
            } else {
                None
            };
            let Some(op) = op else { break };
            let right = self.parse_unary()?;
            let span = Span::new(expr_start(&left), self.i);
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, Diagnostic> {
        self.skip_ws_and_comments();
        if self.eat_punct('!') {
            let start = self.i.saturating_sub(1);
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(expr),
                span: Span::new(start, self.i),
            });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_primary()?;
        loop {
            self.skip_ws_and_comments();
            if self.eat_punct('?') {
                let span = Span::new(expr_start(&expr), self.i);
                expr = Expr::Try {
                    expr: Box::new(expr),
                    span,
                };
                continue;
            }
            if self.eat_punct('.') {
                self.skip_ws_and_comments();
                if self.eat_punct('*') {
                    let span = Span::new(expr_start(&expr), self.i);
                    expr = Expr::StarProj {
                        receiver: Box::new(expr),
                        span,
                    };
                    continue;
                }
                if self.peek() == Some('{') {
                    let items = self.parse_proj_items_block()?;
                    let span = Span::new(expr_start(&expr), self.i);
                    expr = Expr::StructProj {
                        receiver: Box::new(expr),
                        items,
                        span,
                    };
                    continue;
                }
                expr = self.finish_member(expr, PathSep::Dot)?;
                continue;
            }
            if self.eat_punct_seq("::") {
                expr = self.finish_member(expr, PathSep::ColonColon)?;
                continue;
            }
            if self.peek() == Some('(') {
                // Free/call on primary result: `uuid()` after Name
                let args = self.parse_arg_list()?;
                let span = Span::new(expr_start(&expr), self.i);
                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                    span,
                };
                continue;
            }
            break;
        }
        Ok(expr)
    }

    fn finish_member(&mut self, expr: Expr, sep: PathSep) -> Result<Expr, Diagnostic> {
        self.skip_ws_and_comments();
        let name_start = self.i;
        let name = self.expect_any_ident()?;
        let member_span = Span::new(name_start, self.i);
        let mut expr = Expr::Member {
            object: Box::new(expr),
            name,
            sep,
            span: member_span,
        };
        self.skip_ws_and_comments();
        if self.peek() == Some('(') {
            let args = self.parse_arg_list()?;
            let span = Span::new(expr_start(&expr), self.i);
            expr = Expr::Call {
                callee: Box::new(expr),
                args,
                span,
            };
        } else if sep == PathSep::Dot && self.peek() == Some('{') {
            let obj = self.parse_anon_or_typed_fields(None)?;
            let span = Span::new(expr_start(&expr), self.i);
            expr = Expr::Call {
                callee: Box::new(expr),
                args: vec![obj],
                span,
            };
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, Diagnostic> {
        self.skip_ws_and_comments();
        let start = self.i;

        if self.eat_punct('(') {
            let after_paren = self.i;
            if let Some(lambda) = self.try_parse_paren_lambda(start) {
                return Ok(lambda);
            }
            self.i = after_paren;
            let expr = self.parse_expression()?;
            self.expect_punct(')')?;
            return Ok(expr);
        }

        if self.peek() == Some('[') {
            return self.parse_list();
        }

        if self.peek() == Some('{') {
            return self.parse_anon_or_typed_fields(None);
        }

        if let Some(lit) = self.try_parse_literal()? {
            return Ok(Expr::Literal(lit));
        }

        // Identifier: name, typed object `User {`, or lambda `x =>`
        let name = self.expect_any_ident()?;
        self.skip_ws_and_comments();
        if self.eat_punct_seq("=>") {
            let body = self.parse_lambda_body()?;
            return Ok(Expr::Lambda(Lambda {
                params: vec![name],
                body: Box::new(body),
                span: Span::new(start, self.i),
            }));
        }
        if self.peek() == Some('{') {
            // Typed object `User { … }`
            let fields = self.parse_field_init_block()?;
            return Ok(Expr::TypedObject {
                ty: name,
                fields,
                span: Span::new(start, self.i),
            });
        }
        Ok(Expr::Name {
            name,
            span: Span::new(start, self.i),
        })
    }

    /// Try `(a, b) => body` starting after `(`. Restores nothing on failure — caller resets `i`.
    fn try_parse_paren_lambda(&mut self, start: usize) -> Option<Expr> {
        let checkpoint = self.i;
        self.skip_ws_and_comments();
        let mut params = Vec::new();
        if !self.eat_punct(')') {
            loop {
                self.skip_ws_and_comments();
                let ident_start = self.i;
                let Ok(ident) = self.expect_any_ident() else {
                    self.i = checkpoint;
                    return None;
                };
                // Ensure we only consumed an ident (expect_any_ident already did).
                let _ = ident_start;
                params.push(ident);
                self.skip_ws_and_comments();
                if self.eat_punct(')') {
                    break;
                }
                if !self.eat_punct(',') {
                    self.i = checkpoint;
                    return None;
                }
            }
        }
        if !self.eat_punct_seq("=>") {
            self.i = checkpoint;
            return None;
        }
        match self.parse_lambda_body() {
            Ok(body) => Some(Expr::Lambda(Lambda {
                params,
                body: Box::new(body),
                span: Span::new(start, self.i),
            })),
            Err(_) => {
                self.i = checkpoint;
                None
            }
        }
    }

    fn parse_lambda_body(&mut self) -> Result<Expr, Diagnostic> {
        self.skip_ws_and_comments();
        if self.peek() == Some('{') {
            // Patch / anon object body.
            self.parse_anon_or_typed_fields(None)
        } else {
            self.parse_expression()
        }
    }

    fn parse_list(&mut self) -> Result<Expr, Diagnostic> {
        let start = self.i;
        self.expect_punct('[')?;
        let mut items = Vec::new();
        loop {
            self.skip_ws_and_comments();
            if self.eat_punct(']') {
                break;
            }
            items.push(self.parse_expression()?);
            self.skip_ws_and_comments();
            if self.eat_punct(',') {
                continue;
            }
            self.expect_punct(']')?;
            break;
        }
        Ok(Expr::List {
            items,
            span: Span::new(start, self.i),
        })
    }

    fn parse_anon_or_typed_fields(&mut self, ty: Option<String>) -> Result<Expr, Diagnostic> {
        let start = self.i;
        let fields = self.parse_field_init_block()?;
        let span = Span::new(start, self.i);
        match ty {
            Some(ty) => Ok(Expr::TypedObject { ty, fields, span }),
            None => Ok(Expr::AnonObject { fields, span }),
        }
    }

    fn parse_field_init_block(&mut self) -> Result<Vec<FieldInit>, Diagnostic> {
        self.expect_punct('{')?;
        let mut fields = Vec::new();
        loop {
            self.skip_ws_and_comments();
            if self.eat_punct('}') {
                break;
            }
            // Reject `*` here for typed/anon objects (only in projections).
            if self.peek() == Some('*') {
                return Err(Diagnostic::new(
                    "`*` spread is only valid inside `x.{ … }` projections",
                    Span::empty(self.i),
                    Some("use `x.{ *, name: expr }` for spreads"),
                ));
            }
            fields.push(self.parse_field_init()?);
            self.skip_ws_and_comments();
            let _ = self.eat_punct(',');
        }
        Ok(fields)
    }

    fn parse_proj_items_block(&mut self) -> Result<Vec<ProjItem>, Diagnostic> {
        self.expect_punct('{')?;
        let mut items = Vec::new();
        loop {
            self.skip_ws_and_comments();
            if self.eat_punct('}') {
                break;
            }
            if self.eat_punct('*') {
                items.push(ProjItem::Star {
                    span: Span::new(self.i.saturating_sub(1), self.i),
                });
                self.skip_ws_and_comments();
                let _ = self.eat_punct(',');
                continue;
            }
            items.push(ProjItem::Field(self.parse_field_init()?));
            self.skip_ws_and_comments();
            let _ = self.eat_punct(',');
        }
        Ok(items)
    }

    fn parse_field_init(&mut self) -> Result<FieldInit, Diagnostic> {
        let start = self.i;
        let name = self.expect_any_ident()?;
        self.skip_ws_and_comments();

        if self.eat_punct(':') {
            let value = self.parse_expression()?;
            Ok(FieldInit::named(name, value, Span::new(start, self.i)))
        } else {
            Ok(FieldInit::shorthand(name, Span::new(start, self.i)))
        }
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Expr>, Diagnostic> {
        self.expect_punct('(')?;
        let mut args = Vec::new();
        loop {
            self.skip_ws_and_comments();
            if self.eat_punct(')') {
                break;
            }
            args.push(self.parse_expression()?);
            self.skip_ws_and_comments();
            if self.eat_punct(',') {
                continue;
            }
            self.expect_punct(')')?;
            break;
        }
        Ok(args)
    }

    fn try_parse_literal(&mut self) -> Result<Option<Literal>, Diagnostic> {
        self.skip_ws_and_comments();
        if self.eat_ident_value("null") {
            return Ok(Some(Literal::Null));
        }
        if self.eat_ident_value("true") {
            return Ok(Some(Literal::Bool(true)));
        }
        if self.eat_ident_value("false") {
            return Ok(Some(Literal::Bool(false)));
        }
        if self.peek() == Some('"') {
            return Ok(Some(self.parse_string()?));
        }
        if matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            let start = self.i;
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.bump();
            }
            if self.peek() == Some('.') {
                self.bump();
                while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                    self.bump();
                }
                return Ok(Some(Literal::Float(self.src[start..self.i].to_owned())));
            }
            return Ok(Some(Literal::Int(self.src[start..self.i].to_owned())));
        }
        Ok(None)
    }

    fn parse_string(&mut self) -> Result<Literal, Diagnostic> {
        self.expect_punct('"')?;
        let start = self.i;
        while let Some(c) = self.peek() {
            if c == '"' {
                let content = self.src[start..self.i].to_owned();
                self.bump();
                return Ok(Literal::String(content));
            }
            if c == '\\' {
                self.bump();
                let _ = self.bump();
                continue;
            }
            if c == '\n' {
                break;
            }
            self.bump();
        }
        Err(Diagnostic::new(
            "unterminated string literal",
            Span::empty(self.i),
            Some("close the string with `\"`"),
        ))
    }

    // --- lexer helpers (mirrors schema parser style) ---

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

fn expr_start(expr: &Expr) -> usize {
    match expr {
        Expr::Literal(_) => 0,
        Expr::Name { span, .. }
        | Expr::TypedObject { span, .. }
        | Expr::AnonObject { span, .. }
        | Expr::List { span, .. }
        | Expr::Member { span, .. }
        | Expr::Call { span, .. }
        | Expr::Unary { span, .. }
        | Expr::Binary { span, .. }
        | Expr::StarProj { span, .. }
        | Expr::StructProj { span, .. }
        | Expr::Try { span, .. } => span.start,
        Expr::Lambda(l) => l.span.start,
        _ => 0,
    }
}
