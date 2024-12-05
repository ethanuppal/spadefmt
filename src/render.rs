// Copyright (C) 2024 Ethan Uppal.
//
// This file is part of spadefmt.
//
// spadefmt is free software: you can redistribute it and/or modify it under the
// terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version. spadefmt is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
// FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details. You should have received a copy of the GNU General Public License
// along with spadefmt. If not, see <https://www.gnu.org/licenses/>.

use std::fmt;

use spade_ast as ast;
use spade_common::{
    location_info::Loc,
    name::{Identifier, Path},
};
use spade_parser::lexer;

use crate::{
    config::Config,
    format_stream::{CommitableFormatStream, FormatStream},
};

pub struct Context<'stream, 'config> {
    f: CommitableFormatStream<'stream>,
    config: &'config Config,
}

pub trait RenderInContext {
    fn render_in_context(&self, context: &mut Context) -> fmt::Result;
}

macro_rules! can_render {
    ($T:ty: $name:ident) => {
        impl RenderInContext for $T {
            fn render_in_context(&self, context: &mut Context) -> fmt::Result {
                context.$name(self)
            }
        }
    };
}

can_render!(ast::Item: render_item);
can_render!(Loc<ast::Expression>: render_expression);
can_render!(Loc<ast::TypeExpression>: render_type_expression);
can_render!(Loc<ast::TypeParam>: render_type_param);
can_render!(Loc<ast::TraitSpec>: render_trait_spec);

pub type AstParameter =
    (ast::AttributeList, Loc<Identifier>, Loc<ast::TypeSpec>);

can_render!(AstParameter: render_parameter);

pub enum EnclosedRenderStyle {
    Line,
    Tall,
}

impl<'stream, 'config> Context<'stream, 'config> {
    pub fn new<F: FormatStream>(
        f: &'stream mut F, config: &'config Config,
    ) -> Self {
        let f = CommitableFormatStream::new_with_config(
            f,
            config.indent.into(),
            config.max_width.into(),
        );
        Self { f, config }
    }

    pub fn render(&mut self, top: &ast::ModuleBody) -> fmt::Result {
        for (i, item) in top.members.iter().enumerate() {
            if i > 0 {
                self.f.newline()?;
            }
            self.render_item(item)?;
        }
        Ok(())
    }

    pub fn render_item(&mut self, item: &ast::Item) -> fmt::Result {
        match item {
            ast::Item::Unit(unit) => self.render_unit(unit),
            ast::Item::TraitDef(_) => todo!(),
            ast::Item::Type(_) => todo!(),
            ast::Item::Module(module) => self.render_module(module),
            ast::Item::Use(use_statement) => self.render_use(use_statement),
            ast::Item::Config(_) => todo!(),
            ast::Item::ImplBlock(_) => todo!(),
        }
    }

    pub fn render_unit(&mut self, unit: &Loc<ast::Unit>) -> fmt::Result {
        self.render_attribute_list(&unit.head.attributes, true)?;

        match &*unit.head.unit_kind {
            ast::UnitKind::Function => self.f.keyword("fn"),
            ast::UnitKind::Entity => self.f.keyword("entity"),
            ast::UnitKind::Pipeline(depth) => {
                self.f.keyword("pipeline")?;
                self.f.symbol("(")?;
                match &**depth {
                    ast::comptime::MaybeComptime::Raw(raw) => {
                        self.render_type_expression(raw)
                    }
                    ast::comptime::MaybeComptime::Comptime(_) => todo!(),
                }?;
                self.f.symbol(")")
            }
        }?;

        self.f.space()?;
        self.f.identifier(&unit.head.name.0)?;

        if let Some(type_params) = &unit.head.type_params {
            self.render_enclosed_auto(
                Some(lexer::TokenKind::Lt),
                Some(lexer::TokenKind::Gt),
                type_params,
                Some(lexer::TokenKind::Comma),
                false,
                true,
            )?;
        }

        if !self.try_commit(|c| {
            c.render_parameter_list(
                &unit.head.inputs,
                EnclosedRenderStyle::Line,
            )?;
            c.f.space()?;

            if let Some(output_type) = &unit.head.output_type {
                c.f.symbol("->")?;
                c.f.space()?;
                c.render_type_spec(output_type)?;
                c.f.space()?;
            }

            Ok(())
        })? {
            self.render_parameter_list(
                &unit.head.inputs,
                EnclosedRenderStyle::Tall,
            )?;
            self.f.space()?;

            if let Some(output_type) = &unit.head.output_type {
                self.f.symbol("->")?;
                self.f.space()?;
                self.render_type_spec(output_type)?;
                self.f.space()?;
            }
        }

        if !unit.head.where_clauses.is_empty() {
            todo!()
        }

        match &unit.body {
            Some(body) => self.render_expression(body),
            None => self.f.keyword("__builtin__"),
        }?;

        self.f.newline()
    }

    pub fn render_module(&mut self, item: &Loc<ast::Module>) -> fmt::Result {
        self.f.keyword("mod")?;
        self.f.space()?;
        self.f.symbol("{{")?;
        self.f.newline()?;

        self.f.indent()?;
        self.render_module_body(&item.body)?;
        self.f.dedent()?;

        self.f.newline()?;
        self.f.symbol("}}")?;

        Ok(())
    }

    pub fn render_module_body(
        &mut self, body: &Loc<ast::ModuleBody>,
    ) -> fmt::Result {
        for (i, item) in body.members.iter().enumerate() {
            if i > 0 {
                self.f.newline()?;
            }
            self.render_item(item)?;
        }
        Ok(())
    }

    pub fn render_use(
        &mut self, use_statement: &Loc<ast::UseStatement>,
    ) -> fmt::Result {
        let ast::UseStatement { path, alias } = &use_statement.inner;

        self.render_path(path)?;

        if let Some(alias) = alias {
            self.f.space()?;
            self.f.keyword("as")?;
            self.f.space()?;
            self.f.terminal_segment(&alias.0)?;
        }

        self.f.newline()?;

        Ok(())
    }

    pub fn render_path(&mut self, path: &Loc<Path>) -> fmt::Result {
        let segments = &path.0;
        let mut i = 0;

        if segments[0].0 == "lib" {
            self.f.keyword("lib")?;
            i += 1;
        }

        if segments.len() >= 2 {
            while i < segments.len() - 2 {
                self.f.nonterminal_segment(&segments[i].0)?;
                self.f.symbol("::")?;
                i += 1;
            }
        }

        self.f.terminal_segment(&segments[i].0)?;

        Ok(())
    }

    /// Requires: a newline has just been formatted.
    /// Ensures: a newline has been written.
    pub fn render_statement(
        &mut self, statement: &Loc<ast::Statement>,
    ) -> fmt::Result {
        match &**statement {
            ast::Statement::Label(loc) => todo!(),
            ast::Statement::Declaration(vec) => todo!(),
            ast::Statement::Binding(binding) => {
                self.f.keyword("let")?;
                self.f.space()?;
                self.render_pattern(&binding.pattern)?;

                if let Some(ty) = &binding.ty {
                    self.f.symbol(":")?;
                    self.f.space()?;
                    self.render_type_spec(ty)?;
                }

                self.f.space()?;
                self.f.symbol("=")?;
                self.f.space()?;

                self.render_expression(&binding.value)?;
            }
            ast::Statement::PipelineRegMarker(loc, loc1) => {
                self.f.dedent()?;

                self.f.indent()?;
            }
            ast::Statement::Register(loc) => todo!(),
            ast::Statement::Set { target, value } => {
                self.f.keyword("set")?;
                self.f.space()?;
                self.render_expression(target)?;
                self.f.space()?;
                self.f.symbol("=")?;
                self.f.space()?;
                self.render_expression(value)?;
            }
            ast::Statement::Assert(loc) => todo!(),
            ast::Statement::Comptime(comptime_condition) => todo!(),
        }
        self.f.symbol(lexer::TokenKind::Semi.as_str())?;
        self.f.newline()
    }

    pub fn render_expression(
        &mut self, expression: &Loc<ast::Expression>,
    ) -> fmt::Result {
        match &**expression {
            ast::Expression::Identifier(path) => self.render_path(path),
            ast::Expression::IntLiteral(int_literal) => {
                self.f.literal(&int_literal.to_string())
            }
            ast::Expression::BoolLiteral(bool_literal) => {
                self.f.literal(&bool_literal.to_string())
            }
            ast::Expression::BitLiteral(bit_literal) => {
                self.f.literal(match bit_literal {
                    ast::BitLiteral::Low => "LOW",
                    ast::BitLiteral::High => "HIGH",
                    ast::BitLiteral::HighImp => "UNDEF",
                })
            }
            ast::Expression::ArrayLiteral(array_literal) => self
                .render_enclosed_auto(
                    Some(lexer::TokenKind::OpenBracket),
                    Some(lexer::TokenKind::CloseBracket),
                    array_literal,
                    Some(lexer::TokenKind::Comma),
                    false,
                    true,
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
                self.f.symbol("{")?;
                if block.statements.len()
                    + block.result.as_ref().map_or(0, |_| 1)
                    > 0
                {
                    self.f.newline()?;
                    self.f.indent()?;

                    for statement in &block.statements {
                        self.render_statement(statement)?;
                        // automatic newline
                    }

                    if let Some(result) = &block.result {
                        self.render_expression(result)?;
                        self.f.newline()?;
                    }

                    self.f.dedent()?;
                }
                self.f.symbol("}")
            }
            ast::Expression::PipelineReference {
                stage_kw_and_reference_loc,
                stage,
                name,
            } => todo!(),
            ast::Expression::StageValid => todo!(),
            ast::Expression::StageReady => todo!(),
            ast::Expression::Comptime(expression) => todo!(),
        }
    }

    pub fn render_pattern(
        &mut self, pattern: &Loc<ast::Pattern>,
    ) -> fmt::Result {
        match &**pattern {
            ast::Pattern::Integer(int_literal) => {
                self.f.literal(&int_literal.to_string())
            }
            ast::Pattern::Bool(bool_literal) => {
                self.f.literal(&bool_literal.to_string())
            }
            ast::Pattern::Path(path) => self.render_path(path),
            ast::Pattern::Tuple(vec) => todo!(),
            ast::Pattern::Array(vec) => todo!(),
            ast::Pattern::Type(loc, loc1) => todo!(),
        }
    }

    pub fn render_type_expression(
        &mut self, type_expression: &Loc<ast::TypeExpression>,
    ) -> fmt::Result {
        match &**type_expression {
            ast::TypeExpression::TypeSpec(type_spec) => {
                self.render_type_spec(type_spec)
            }
            ast::TypeExpression::Integer(value) => {
                self.f.literal(&value.to_string())
            }
            ast::TypeExpression::ConstGeneric(expression) => {
                self.render_expression(expression)
            }
        }
    }

    pub fn render_type_spec(
        &mut self, type_spec: &Loc<ast::TypeSpec>,
    ) -> fmt::Result {
        match &**type_spec {
            ast::TypeSpec::Tuple(elements) => {
                self.f.symbol("(")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.f.symbol(",")?;
                        self.f.space()?;
                    }
                    self.render_type_spec(elem)?;
                }
                self.f.symbol(")")?;
            }
            ast::TypeSpec::Array { inner, size } => {
                self.f.symbol("[")?;
                self.render_type_spec(inner)?;
                self.f.symbol(";")?;
                self.f.space()?;
                self.render_type_expression(size)?;
                self.f.symbol("]")?;
            }
            ast::TypeSpec::Named(path, type_params) => {
                // TODO: somehow use f.type_name()
                self.render_path(path)?;
                if let Some(params) = type_params {
                    self.f.symbol("<")?;
                    for (i, param) in params.iter().enumerate() {
                        if i > 0 {
                            self.f.symbol(",")?;
                            self.f.space()?;
                        }
                        self.render_type_expression(param)?;
                    }
                    self.f.symbol(">")?;
                }
            }
            ast::TypeSpec::Unit(_) => self.f.symbol("()")?,
            ast::TypeSpec::Inverted(inner) => {
                self.f.keyword("inv")?;
                self.f.space()?;
                self.render_type_spec(inner)?;
            }
            ast::TypeSpec::Wire(inner) => {
                self.f.keyword("wire")?;
                self.f.space()?;
                self.render_type_spec(inner)?;
            }
            ast::TypeSpec::Wildcard => self.f.symbol("_")?,
        }

        Ok(())
    }

    pub fn render_type_param(
        &mut self, type_param: &Loc<ast::TypeParam>,
    ) -> fmt::Result {
        match &**type_param {
            ast::TypeParam::TypeName { name, traits } => {
                self.f.identifier(&name.0)?;
                if !traits.is_empty() {
                    self.f.symbol(":")?;
                    self.f.space()?;
                    self.render_enclosed_auto(
                        None,
                        None,
                        traits,
                        Some(lexer::TokenKind::Plus),
                        true,
                        false,
                    )?;
                }
                Ok(())
            }
            ast::TypeParam::TypeWithMeta { meta, name } => todo!(),
        }
    }

    pub fn render_trait_spec(
        &mut self, trait_spec: &Loc<ast::TraitSpec>,
    ) -> fmt::Result {
        self.render_path(&trait_spec.path)?;
        if let Some(type_params) = &trait_spec.type_params {
            self.render_enclosed_auto(
                Some(lexer::TokenKind::Lt),
                Some(lexer::TokenKind::Gt),
                type_params,
                Some(lexer::TokenKind::Comma),
                false,
                true,
            )?;
        }

        Ok(())
    }

    pub fn render_attribute(
        &mut self, attribute: &Loc<ast::Attribute>,
    ) -> fmt::Result {
        match &**attribute {
            ast::Attribute::Optimize { passes } => todo!(),
            ast::Attribute::NoMangle => self.f.attribute("no_mangle"),
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

    pub fn render_attribute_list(
        &mut self, attribute_list: &ast::AttributeList, always_newline: bool,
    ) -> fmt::Result {
        match attribute_list.0.len() {
            0 => Ok(()),
            1 => {
                self.render_attribute(&attribute_list.0[0])?;
                if always_newline {
                    self.f.newline()
                } else {
                    self.f.space()
                }
            }
            _ => attribute_list.0.iter().try_for_each(|attribute| {
                self.render_attribute(attribute)?;
                self.f.newline()
            }),
        }
    }

    pub fn render_parameter(
        &mut self, parameter: &AstParameter,
    ) -> fmt::Result {
        self.render_attribute_list(&parameter.0, false)?;
        self.f.identifier(&parameter.1 .0)?;
        self.f.symbol(":")?;
        self.f.space()?;
        self.render_type_spec(&parameter.2)
    }

    pub fn render_parameter_list(
        &mut self, parameter_list: &Loc<ast::ParameterList>,
        enclosed_render_style: EnclosedRenderStyle,
    ) -> fmt::Result {
        self.render_enclosed(
            Some(lexer::TokenKind::OpenParen),
            Some(lexer::TokenKind::CloseParen),
            &parameter_list.args,
            Some(lexer::TokenKind::Comma),
            enclosed_render_style,
            false,
            true,
        )
    }

    /// Use [`Self::render_enclosed_auto`] unless you need a specific
    /// [`EnclosedRenderStyle`].
    pub fn render_enclosed<R: RenderInContext>(
        &mut self, open_symbol: Option<lexer::TokenKind>,
        close_symbol: Option<lexer::TokenKind>, enclosed: &[R],
        deliminter_symbol: Option<lexer::TokenKind>,
        enclosed_render_style: EnclosedRenderStyle, even_delimeter: bool,
        trailing_delimiter: bool,
    ) -> fmt::Result {
        if let Some(open_symbol) = open_symbol {
            self.f.symbol(open_symbol.as_str())?;
        }
        match enclosed_render_style {
            EnclosedRenderStyle::Line => {
                for (i, can_render) in enclosed.iter().enumerate() {
                    if i > 0 {
                        if let Some(deliminter_symbol) = &deliminter_symbol {
                            if even_delimeter {
                                self.f.space()?;
                            }
                            self.f.symbol(deliminter_symbol.as_str())?;
                        }
                        self.f.space()?;
                    }
                    can_render.render_in_context(self)?;
                }
            }
            EnclosedRenderStyle::Tall => {
                self.f.indent()?;
                self.f.newline()?;
                for (i, can_render) in enclosed.iter().enumerate() {
                    if i > 0 {
                        if let Some(deliminter_symbol) = &deliminter_symbol {
                            if even_delimeter {
                                self.f.space()?;
                            }
                            self.f.symbol(deliminter_symbol.as_str())?;
                        }
                        self.f.newline()?;
                    }
                    can_render.render_in_context(self)?;
                }
                if trailing_delimiter {
                    if let Some(deliminter_symbol) = &deliminter_symbol {
                        self.f.symbol(deliminter_symbol.as_str())?;
                    }
                }
                self.f.newline()?;
                self.f.dedent()?;
            }
        }
        if let Some(close_symbol) = close_symbol {
            self.f.symbol(close_symbol.as_str())?;
        }
        Ok(())
    }

    fn try_commit<F: Fn(&mut Self) -> fmt::Result>(
        &mut self, try_commit: F,
    ) -> Result<bool, fmt::Error> {
        let mark = self.f.push_mark();
        self.f.disable_commit();
        try_commit(self)?;
        if self.f.width_limit_failure() {
            self.f.return_to_mark(mark);
            Ok(false)
        } else {
            self.f.return_to_mark(mark);
            try_commit(self)?;
            Ok(true)
        }
    }

    fn render_enclosed_auto<R: RenderInContext>(
        &mut self, open_symbol: Option<lexer::TokenKind>,
        close_symbol: Option<lexer::TokenKind>, enclosed: &[R],
        deliminter_symbol: Option<lexer::TokenKind>, even_delimeter: bool,
        trailing_delimiter: bool,
    ) -> fmt::Result {
        if self.try_commit(|c| {
            c.render_enclosed(
                open_symbol.clone(),
                close_symbol.clone(),
                enclosed,
                deliminter_symbol.clone(),
                EnclosedRenderStyle::Line,
                even_delimeter,
                trailing_delimiter,
            )
        })? {
            Ok(())
        } else {
            self.render_enclosed(
                open_symbol,
                close_symbol,
                enclosed,
                deliminter_symbol,
                EnclosedRenderStyle::Tall,
                even_delimeter,
                trailing_delimiter,
            )
        }
    }
}
