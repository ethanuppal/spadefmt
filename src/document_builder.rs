// Copyright (C) 2024 Ethan Uppal.
//
// This file is part of spadefmt.
//
// spadefmt is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version. spadefmt is distributed in the hope that it will be
// useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General
// Public License for more details. You should have received a copy of the GNU
// General Public License along with spadefmt. If not, see <https://www.gnu.org/licenses/>.

use std::cell::RefCell;

use spade_ast as ast;
use spade_common::{
    location_info::Loc,
    name::{Identifier, Path},
};
use spade_parser::lexer;

use crate::document::{Document, DocumentIdx, InternedDocumentStore};

pub struct DocumentBuilder {
    indent: isize,
    inner: RefCell<InternedDocumentStore>,
}

pub trait BuildAsDocument {
    fn build(&self, builder: &DocumentBuilder) -> DocumentIdx;
}

macro_rules! can_build {
    ($T:ty: $name:ident) => {
        impl BuildAsDocument for $T {
            fn build(
                &self, builder: &DocumentBuilder,
            ) -> $crate::document::DocumentIdx {
                builder.$name(self)
            }
        }

        impl BuildAsDocument for Loc<$T> {
            fn build(
                &self, builder: &DocumentBuilder,
            ) -> $crate::document::DocumentIdx {
                builder.$name(self)
            }
        }
    };
}

can_build!(ast::Item: build_item);
can_build!(Loc<ast::Expression>: build_expression);
can_build!(Loc<ast::TypeExpression>: build_type_expression);
can_build!(Loc<ast::TypeParam>: build_type_param);
can_build!(Loc<ast::TraitSpec>: build_trait_spec);

pub type AstParameter =
    (ast::AttributeList, Loc<Identifier>, Loc<ast::TypeSpec>);

can_build!(AstParameter: build_parameter);

impl DocumentBuilder {
    pub fn new(indent: isize) -> Self {
        Self {
            indent,
            inner: Default::default(),
        }
    }

    pub fn build_root(
        self, root: &ast::ModuleBody,
    ) -> (InternedDocumentStore, DocumentIdx) {
        let mut list = vec![];
        for (i, item) in root.members.iter().enumerate() {
            if i > 0 {
                list.push(self.newline());
            }
            list.push(self.build_item(item));
        }
        let idx = self.list(list);
        (self.inner.take(), idx)
    }

    pub fn build_item(&self, item: &ast::Item) -> DocumentIdx {
        match item {
            ast::Item::Unit(unit) => self.build_unit(unit),
            ast::Item::TraitDef(_) => todo!(),
            ast::Item::Type(_) => todo!(),
            ast::Item::ExternalMod(_) => todo!(),
            ast::Item::Module(module) => self.build_module(module),
            ast::Item::Use(use_statement) => self.build_use(use_statement),
            ast::Item::ImplBlock(_) => todo!(),
        }
    }

    pub fn build_unit(&self, unit: &Loc<ast::Unit>) -> DocumentIdx {
        let mut list = vec![];

        list.push(self.build_attribute_list(&unit.head.attributes, true));

        list.push(match &*unit.head.unit_kind {
            ast::UnitKind::Function => self.text("fn"),
            ast::UnitKind::Entity => self.text("entity"),
            ast::UnitKind::Pipeline(depth) => self.list([
                self.text("pipeline("),
                self.build_type_expression(depth),
                self.text(")"),
            ]),
        });

        list.push(self.text(format!(" {}", unit.head.name)));

        if let Some(type_params) = &unit.head.type_params {
            list.push(self.group(
                lexer::TokenKind::Lt,
                &type_params.inner,
                lexer::TokenKind::Comma,
                lexer::TokenKind::Gt,
            ));
        }

        list.push(self.build_parameter_list(&unit.head.inputs));

        if let Some((_, output_type)) = &unit.head.output_type {
            list.extend([self.text(" -> "), self.build_type_spec(output_type)]);
        }

        if !unit.head.where_clauses.is_empty() {
            todo!()
        }

        list.push(match &unit.body {
            Some(body) => self.build_expression(body),
            None => self.text(";"),
        });

        self.list(list)
    }

    pub fn build_module(&self, item: &Loc<ast::Module>) -> DocumentIdx {
        self.list([
            self.text(format!("mod {} {{", item.name)),
            self.newline(),
            self.nest(self.build_module_body(&item.body), self.indent),
            self.newline(),
            self.text("}}"),
        ])
    }

    pub fn build_module_body(
        &self, body: &Loc<ast::ModuleBody>,
    ) -> DocumentIdx {
        let mut list = vec![];
        for (i, item) in body.members.iter().enumerate() {
            if i > 0 {
                list.push(self.newline());
            }
            list.push(self.build_item(item));
        }
        self.list(list)
    }

    pub fn build_use(
        &self, use_statement: &Loc<ast::UseStatement>,
    ) -> DocumentIdx {
        let ast::UseStatement { path, alias } = &use_statement.inner;

        let mut line = vec![self.text("use "), self.build_path(path)];

        if let Some(alias) = alias {
            line.push(self.text(format!(" as {}", alias)));
        }

        line.push(self.newline());
        self.list(line)
    }

    pub fn build_path(&self, path: &Loc<Path>) -> DocumentIdx {
        self.text(
            path.inner
                .0
                .iter()
                .map(|component| component.to_string())
                .collect::<Vec<_>>()
                .join("::"),
        )
    }

    pub fn build_statement(
        &self, statement: &Loc<ast::Statement>,
    ) -> DocumentIdx {
        match &**statement {
            ast::Statement::Label(loc) => todo!(),
            ast::Statement::Declaration(vec) => todo!(),
            ast::Statement::Binding(binding) => {
                let mut list = vec![
                    self.text("let "),
                    self.build_pattern(&binding.pattern),
                ];

                if let Some(ty) = &binding.ty {
                    list.extend([self.text(": "), self.build_type_spec(ty)]);
                }

                list.push(self.text(" = "));
                list.push(self.build_expression(&binding.value));

                self.list(list)
            }
            ast::Statement::PipelineRegMarker(loc, loc1) => {
                todo!()
            }
            ast::Statement::Register(loc) => todo!(),
            ast::Statement::Set { target, value } => self.list([
                self.text("set "),
                self.build_expression(target),
                self.text(" = "),
                self.build_expression(value),
            ]),
            ast::Statement::Assert(loc) => todo!(),
        }
    }

    pub fn build_expression(
        &self, expression: &Loc<ast::Expression>,
    ) -> DocumentIdx {
        match &**expression {
            ast::Expression::Identifier(path) => self.build_path(path),
            ast::Expression::IntLiteral(int_literal) => {
                self.text(int_literal.to_string())
            }
            ast::Expression::BoolLiteral(bool_literal) => {
                self.text(bool_literal.to_string())
            }
            ast::Expression::BitLiteral(bit_literal) => {
                self.text(match bit_literal {
                    ast::BitLiteral::Low => "LOW",
                    ast::BitLiteral::High => "HIGH",
                    ast::BitLiteral::HighImp => "UNDEF",
                })
            }
            ast::Expression::ArrayLiteral(array_literal) => self.group(
                lexer::TokenKind::OpenBracket,
                array_literal,
                lexer::TokenKind::Comma,
                lexer::TokenKind::CloseBracket,
            ),
            ast::Expression::ArrayShorthandLiteral(loc, loc1) => todo!(),
            ast::Expression::Index(loc, loc1) => todo!(),
            ast::Expression::RangeIndex { target, start, end } => todo!(),
            ast::Expression::TupleLiteral(vec) => todo!(),
            ast::Expression::TupleIndex(loc, loc1) => todo!(),
            ast::Expression::FieldAccess(loc, loc1) => todo!(),
            ast::Expression::CreatePorts => todo!(),
            ast::Expression::Call {
                kind,
                callee,
                args,
                turbofish,
            } => todo!(),
            ast::Expression::MethodCall {
                target,
                name,
                args,
                kind,
                turbofish,
            } => todo!(),
            ast::Expression::If(loc, loc1, loc2) => todo!(),
            ast::Expression::Match(loc, loc1) => todo!(),
            ast::Expression::UnaryOperator(unary_operator, loc) => {
                todo!()
            }
            ast::Expression::BinaryOperator(loc, loc1, loc2) => todo!(),
            ast::Expression::Block(block) => {
                let mut list = vec![self.token(lexer::TokenKind::OpenBrace)];
                if block.statements.len()
                    + block.result.as_ref().map_or(0, |_| 1)
                    > 0
                {
                    list.push(self.newline());

                    let mut nest = vec![];

                    for statement in &block.statements {
                        nest.push(self.build_statement(statement));
                        nest.push(self.newline());
                    }

                    if let Some(result) = &block.result {
                        nest.push(self.build_expression(result));
                        nest.push(self.newline());
                    }

                    list.push(self.nest(self.list(nest), self.indent));
                }
                list.push(self.token(lexer::TokenKind::CloseBrace));

                self.list(list)
            }
            ast::Expression::PipelineReference {
                stage_kw_and_reference_loc,
                stage,
                name,
            } => todo!(),
            ast::Expression::TypeLevelIf(loc, loc1, loc2) => todo!(),
            ast::Expression::StageValid => todo!(),
            ast::Expression::StageReady => todo!(),
        }
    }

    pub fn build_pattern(&self, pattern: &Loc<ast::Pattern>) -> DocumentIdx {
        match &**pattern {
            ast::Pattern::Integer(int_literal) => {
                self.text(int_literal.to_string())
            }
            ast::Pattern::Bool(bool_literal) => {
                self.text(bool_literal.to_string())
            }
            ast::Pattern::Path(path) => self.build_path(path),
            ast::Pattern::Tuple(vec) => todo!(),
            ast::Pattern::Array(vec) => todo!(),
            ast::Pattern::Type(loc, loc1) => todo!(),
        }
    }

    pub fn build_type_expression(
        &self, type_expression: &Loc<ast::TypeExpression>,
    ) -> DocumentIdx {
        match &**type_expression {
            ast::TypeExpression::TypeSpec(type_spec) => {
                self.build_type_spec(type_spec)
            }
            ast::TypeExpression::Integer(value) => self.text(value.to_string()),
            ast::TypeExpression::ConstGeneric(expression) => {
                self.build_expression(expression)
            }
        }
    }

    pub fn build_type_spec(
        &self, type_spec: &Loc<ast::TypeSpec>,
    ) -> DocumentIdx {
        match &**type_spec {
            ast::TypeSpec::Tuple(elements) => self.group(
                lexer::TokenKind::OpenParen,
                elements,
                lexer::TokenKind::Comma,
                lexer::TokenKind::CloseParen,
            ),
            ast::TypeSpec::Array { inner, size } => self.list([
                self.text("["),
                self.build_type_expression(inner),
                self.text("; "),
                self.build_type_expression(size),
                self.text("]"),
            ]),
            ast::TypeSpec::Named(path, type_params) => {
                let mut list = vec![self.build_path(path)];
                if let Some(params) = type_params {
                    list.push(self.group(
                        lexer::TokenKind::Lt,
                        &params.inner,
                        lexer::TokenKind::Comma,
                        lexer::TokenKind::Gt,
                    ));
                }
                self.list(list)
            }
            ast::TypeSpec::Inverted(inner) => self
                .list([self.text("inv "), self.build_type_expression(inner)]),
            ast::TypeSpec::Wire(inner) => {
                self.list([self.text("&"), self.build_type_expression(inner)])
            }
            ast::TypeSpec::Wildcard => self.text("_"),
        }
    }

    pub fn build_type_param(
        &self, type_param: &Loc<ast::TypeParam>,
    ) -> DocumentIdx {
        match &**type_param {
            ast::TypeParam::TypeName { name, traits } => {
                let mut list = vec![self.text(name.to_string())];
                if !traits.is_empty() {
                    list.extend([
                        self.text(": "),
                        self.group(None, traits, lexer::TokenKind::Plus, None),
                    ])
                }
                self.list(list)
            }
            ast::TypeParam::TypeWithMeta { meta, name } => todo!(),
        }
    }

    pub fn build_trait_spec(
        &self, trait_spec: &Loc<ast::TraitSpec>,
    ) -> DocumentIdx {
        let mut list = vec![self.build_path(&trait_spec.path)];
        if let Some(type_params) = &trait_spec.type_params {
            list.push(self.group(
                lexer::TokenKind::Lt,
                &type_params.inner,
                lexer::TokenKind::Comma,
                lexer::TokenKind::Gt,
            ));
        }
        self.list(list)
    }

    pub fn build_attribute(
        &self, attribute: &Loc<ast::Attribute>,
    ) -> DocumentIdx {
        match &**attribute {
            ast::Attribute::Optimize { passes } => todo!(),
            ast::Attribute::NoMangle { all } => self.text(format!(
                "#[no_mangle{}]",
                if *all { "(all)" } else { "" }
            )),
            ast::Attribute::Fsm { state } => todo!(),
            ast::Attribute::WalTraceable {
                suffix,
                uses_clk,
                uses_rst,
            } => todo!(),
            ast::Attribute::WalTrace { clk, rst } => todo!(),
            ast::Attribute::WalSuffix { suffix } => todo!(),
        }
    }

    pub fn build_attribute_list(
        &self, attribute_list: &ast::AttributeList, always_newline: bool,
    ) -> DocumentIdx {
        self.list(match attribute_list.0.len() {
            0 => vec![],
            1 => vec![
                self.build_attribute(&attribute_list.0[0]),
                if always_newline {
                    self.newline()
                } else {
                    self.text(" ")
                },
            ],
            _ => {
                let mut list = vec![];
                for attribute in &attribute_list.0 {
                    list.extend([
                        self.build_attribute(attribute),
                        self.newline(),
                    ]);
                }
                list
            }
        })
    }

    pub fn build_parameter(&self, parameter: &AstParameter) -> DocumentIdx {
        self.list([
            self.build_attribute_list(&parameter.0, false),
            self.text(format!("{}: ", parameter.1)),
            self.build_type_spec(&parameter.2),
        ])
    }

    pub fn build_parameter_list(
        &self, parameter_list: &Loc<ast::ParameterList>,
    ) -> DocumentIdx {
        self.group(
            lexer::TokenKind::OpenParen,
            &parameter_list.args,
            lexer::TokenKind::Comma,
            lexer::TokenKind::CloseParen,
        )
    }

    fn newline(&self) -> DocumentIdx {
        self.inner.borrow_mut().add(Document::Newline)
    }

    fn text(&self, text: impl Into<String>) -> DocumentIdx {
        self.inner.borrow_mut().add(Document::Text(text.into()))
    }

    fn token(&self, text: lexer::TokenKind) -> DocumentIdx {
        self.text(text.as_str())
    }

    fn nest(&self, body: DocumentIdx, by: isize) -> DocumentIdx {
        self.inner.borrow_mut().add(Document::Nest(body, by))
    }

    fn flatten(&self, body: DocumentIdx) -> DocumentIdx {
        self.inner.borrow_mut().add(Document::Flatten(body))
    }

    fn try_catch(
        &self, try_body: DocumentIdx, catch_body: DocumentIdx,
    ) -> DocumentIdx {
        self.inner
            .borrow_mut()
            .add(Document::TryCatch(try_body, catch_body))
    }

    fn list(&self, list: impl IntoIterator<Item = DocumentIdx>) -> DocumentIdx {
        self.inner
            .borrow_mut()
            .add(Document::List(list.into_iter().collect()))
    }

    fn group<'a, B: BuildAsDocument + 'a>(
        &self, open: impl Into<Option<lexer::TokenKind>>,
        contents: impl IntoIterator<Item = &'a B>,
        between: impl Into<Option<lexer::TokenKind>>,
        close: impl Into<Option<lexer::TokenKind>>,
    ) -> DocumentIdx {
        let open = open.into();
        let between = between.into();
        let close = close.into();

        let mut list = vec![];
        for (i, item) in contents
            .into_iter()
            .map(|item| item.build(self))
            .enumerate()
        {
            if i > 0 {
                if let Some(ref between) = between {
                    list.push(self.token(between.clone()));
                }
            }
            list.push(item);
            if i > 0 {
                list.push(self.newline());
            }
        }
        let doc_contents = self.list(list);
        // try to flatten, otherwise nest
        let try_catch = self.try_catch(
            self.flatten(doc_contents),
            self.list([
                self.newline(),
                self.nest(doc_contents, self.indent),
                self.newline(),
            ]),
        );
        let mut main_list = vec![];
        if let Some(open) = open {
            main_list.push(self.token(open));
        }
        main_list.push(try_catch);
        if let Some(close) = close {
            main_list.push(self.token(close));
        }
        self.list(main_list)
    }
}
