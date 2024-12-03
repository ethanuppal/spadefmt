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

use std::{cell::RefCell, fmt};

use logos::Source;
use spade_ast as ast;
use spade_common::{location_info::Loc, name::Path};

use crate::{
    config::Config,
    format_stream::{CommitableFormatStream, FormatStream},
};

pub struct Context<'stream, 'config> {
    f: CommitableFormatStream<'stream>,
    config: &'config Config,
}

impl<'stream, 'config> Context<'stream, 'config> {
    pub fn new<F: FormatStream>(
        f: &'stream mut F, config: &'config Config,
    ) -> Self {
        let f = CommitableFormatStream::indenting_by(config.indent.into(), f);
        Self { f, config }
    }

    pub fn render(&mut self, top: &ast::ModuleBody) -> fmt::Result {
        for item in &top.members {
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
        self.f.enable_commit();
        // TODO: figure out good way to decide when to dedent vs. leave
        // `reg` alone

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
        self.f.symbol("(")?;
        self.f.symbol(")")?;
        self.f.newline()?;

        self.f.disable_commit();

        Ok(())
    }

    pub fn render_module(&mut self, item: &Loc<ast::Module>) -> fmt::Result {
        self.f.enable_commit();

        self.f.keyword("mod")?;
        self.f.space()?;
        self.f.symbol("{{")?;
        self.f.newline()?;

        self.f.indent()?;
        self.render_module_body(&item.body)?;
        self.f.dedent()?;

        self.f.newline()?;
        self.f.symbol("}}")?;

        self.f.disable_commit();

        Ok(())
    }

    pub fn render_module_body(
        &mut self, body: &Loc<ast::ModuleBody>,
    ) -> fmt::Result {
        for item in &body.members {
            self.render_item(item)?;
        }
        Ok(())
    }

    pub fn render_use(
        &mut self, use_statement: &Loc<ast::UseStatement>,
    ) -> fmt::Result {
        let ast::UseStatement { path, alias } = &use_statement.inner;

        self.f.enable_commit();

        self.render_path(path)?;

        if let Some(alias) = alias {
            self.f.space()?;
            self.f.keyword("as")?;
            self.f.space()?;
            self.f.terminal_segment(&alias.0)?;
        }

        self.f.newline()?;
        self.f.disable_commit();

        Ok(())
    }

    pub fn render_path(&mut self, path: &Loc<Path>) -> fmt::Result {
        self.f.enable_commit();

        let segments = &path.0;
        let mut i = 0;

        if segments[0].0 == "lib" {
            self.f.keyword("lib")?;
            i += 1;
        }

        while i < segments.len() - 2 {
            self.f.nonterminal_segment(&segments[i].0)?;
            self.f.symbol("::")?;
            i += 1;
        }

        self.f.terminal_segment(&segments[i].0)?;

        self.f.disable_commit();

        Ok(())
    }

    pub fn render_type_expression(
        &mut self, type_expression: &Loc<ast::TypeExpression>,
    ) -> fmt::Result {
        self.f.enable_commit();

        match &**type_expression {
            ast::TypeExpression::TypeSpec(type_spec) => {
                self.render_type_spec(type_spec)?
            }
            ast::TypeExpression::Integer(value) => {
                self.f.literal(&value.to_string())?
            }
            ast::TypeExpression::ConstGeneric(_) => todo!(),
        };

        self.f.disable_commit();

        Ok(())
    }

    pub fn render_type_spec(
        &mut self, type_spec: &Loc<ast::TypeSpec>,
    ) -> fmt::Result {
        self.f.enable_commit();

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

        self.f.disable_commit();

        Ok(())
    }
}
