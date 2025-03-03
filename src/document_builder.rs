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

use spade::lexer;
use type_sitter::{HasChildren, TreeCursor};
use type_sitter_spade as ast;

use crate::document::{Document, DocumentIdx, InternedDocumentStore};

pub struct DocumentBuilder {
    indent: isize,
    inner: RefCell<InternedDocumentStore>,
}

pub trait BuildAsDocument {
    fn build(&self, builder: &DocumentBuilder) -> DocumentIdx;
}

impl BuildAsDocument for DocumentIdx {
    fn build(&self, _builder: &DocumentBuilder) -> DocumentIdx {
        *self
    }
}

macro_rules! can_build {
    ($T:ty: $name:ident) => {
        impl BuildAsDocument for $T {
            fn build(
                &self,
                builder: &DocumentBuilder,
            ) -> $crate::document::DocumentIdx {
                builder.$name(self)
            }
        }
    };
}

//can_build!(ast::Item: build_item);
//can_build!(ast::Expression: build_expression);
//can_build!(ast::TypeExpression: build_type_expression);
//can_build!(ast::TypeParam: build_type_param);
//can_build!(ast::TraitSpec: build_trait_spec);
//can_build!(ast::NamedArgument: build_named_argument);
//can_build!(ast::Pattern: build_pattern);

//pub type AstParameter =
//    (ast::AttributeList, Loc<Identifier>, Loc<ast::TypeSpec>);
//
//can_build!(AstParameter: build_parameter);
//
//pub type EnumVariant = (Loc<Identifier>, Option<Loc<ast::ParameterList>>);
//
//can_build!(EnumVariant: build_enum_variant);

impl DocumentBuilder {
    pub fn new(indent: isize) -> Self {
        Self {
            indent,
            inner: Default::default(),
        }
    }

    pub fn build_root<'a>(
        &self,
        root: &ast::SourceFile<'a>,
        mut cursor: TreeCursor<'a>,
    ) -> (InternedDocumentStore, DocumentIdx) {
        let mut list = vec![];
        let items = root
            .items(&mut cursor)
            .flatten()
            .enumerate()
            .collect::<Vec<_>>();
        for (i, item) in items {
            if i > 0 {
                list.push(self.newline());
            }
            list.push(self.build_item(&item, &mut cursor));
        }
        let idx = self.list(list);
        (self.inner.take(), idx)
    }

    pub fn build_item<'a>(
        &self,
        item: &ast::Item<'a>,
        cursor: &mut TreeCursor<'a>,
    ) -> DocumentIdx {
        let mut children = item.children(cursor).into_iter().flatten();
        let mut attributes = vec![];
        while let Some(next) = children.next() {
            let Some(attribute) = next.as_attribute() else {
                break;
            };
            attributes.push(attribute);
        }

        use ast::anon_unions::Attribute_EnumDefinition_ExternUnitDeclaration_Impl_Mod_StructDefinition_Trait_UnitDefinition_Use as ItemEnum;
        match children.next().expect("Missing item after attributes") {
            //ItemEnum::UnitDefinition(unit) => self.build_unit(unit),
            //ast::Item::TraitDef(_) => todo!(),
            //ast::Item::Type(type_declaration) => {
            //    self.build_type_declaration(type_declaration)
            //}
            //ast::Item::ExternalMod(_) => todo!(),
            //ast::Item::Module(module) => self.build_module(module),
            //ast::Item::Use(use_statement) => self.build_use(use_statement),
            //ast::Item::ImplBlock(impl_block) => {
            //    self.build_impl_block(impl_block)
            //}
            _ => todo!(),
        }
    }

    //pub fn build_unit(&self, unit: &Loc<ast::Unit>) -> DocumentIdx {
    //    let mut list = vec![];
    //
    //    list.push(self.build_attribute_list(&unit.head.attributes, true));
    //
    //    list.push(match &*unit.head.unit_kind {
    //        ast::UnitKind::Function => self.text("fn"),
    //        ast::UnitKind::Entity => self.text("entity"),
    //        ast::UnitKind::Pipeline(depth) => self.list([
    //            self.text("pipeline("),
    //            self.build_type_expression(depth),
    //            self.text(")"),
    //        ]),
    //    });
    //
    //    list.push(self.text(format!(" {}", unit.head.name)));
    //
    //    if let Some(type_params) = &unit.head.type_params {
    //        list.push(self.group(
    //            lexer::TokenKind::Lt.as_str(),
    //            &type_params.inner,
    //            lexer::TokenKind::Comma,
    //            lexer::TokenKind::Gt.as_str(),
    //        ));
    //    }
    //
    //    let parameter_list_doc = self.build_parameter_list(&unit.head.inputs);
    //    let parameter_open = self.token(lexer::TokenKind::OpenParen);
    //    let parameter_close = self.token(lexer::TokenKind::CloseParen);
    //
    //    let output_type_doc = if let Some((_, output_type)) =
    //        &unit.head.output_type
    //    {
    //        self.list([self.text(" -> "), self.build_type_spec(output_type)])
    //    } else {
    //        self.list([])
    //    };
    //
    //    list.push(self.try_catch(
    //        self.list([
    //            parameter_open,
    //            parameter_list_doc.0,
    //            parameter_close,
    //            self.flatten(output_type_doc),
    //        ]),
    //        self.try_catch(
    //            self.list([
    //                parameter_open,
    //                parameter_list_doc.0,
    //                parameter_close,
    //                output_type_doc,
    //            ]),
    //            self.list([
    //                parameter_open,
    //                parameter_list_doc.1,
    //                parameter_close,
    //                output_type_doc,
    //            ]),
    //        ),
    //    ));
    //
    //    if !unit.head.where_clauses.is_empty() {
    //        todo!()
    //    }
    //
    //    list.push(match &unit.body {
    //        Some(body) => {
    //            self.list([self.text(" "), self.build_expression(body)])
    //        }
    //        None => self.text(";"),
    //    });
    //
    //    self.list(list)
    //}
    //
    //pub fn build_type_declaration(
    //    &self,
    //    type_declaration: &Loc<ast::TypeDeclaration>,
    //) -> DocumentIdx {
    //    match &type_declaration.kind {
    //        ast::TypeDeclKind::Enum(enum_decl) => {
    //            let mut list = vec![self.text("enum ")];
    //            list.push(self.text(enum_decl.name.to_string()));
    //            if let Some(generic_args) = &type_declaration.generic_args {
    //                list.push(self.group(
    //                    lexer::TokenKind::Lt.as_str(),
    //                    &generic_args.inner,
    //                    lexer::TokenKind::Comma,
    //                    lexer::TokenKind::Gt.as_str(),
    //                ));
    //            }
    //            let options_doc =
    //                self.group_raw(&enum_decl.options,
    // lexer::TokenKind::Comma);            list.extend([
    //                self.text(" {"),
    //                self.try_catch(
    //                    self.list([
    //                        self.text(" "),
    //                        options_doc.0,
    //                        self.text(" "),
    //                    ]),
    //                    options_doc.1,
    //                ),
    //                self.text("}"),
    //            ]);
    //            self.list(list)
    //        }
    //        ast::TypeDeclKind::Struct(struct_decl) => {
    //            let mut list = vec![self.text("struct ")];
    //            if struct_decl.is_port() {
    //                list.push(self.text("port "));
    //            }
    //            list.push(self.text(struct_decl.name.to_string()));
    //            if let Some(generic_args) = &type_declaration.generic_args {
    //                list.push(self.group(
    //                    lexer::TokenKind::Lt.as_str(),
    //                    &generic_args.inner,
    //                    lexer::TokenKind::Comma,
    //                    lexer::TokenKind::Gt.as_str(),
    //                ));
    //            }
    //            let parameter_list_doc =
    //                self.build_parameter_list(&struct_decl.members);
    //            list.extend([
    //                self.text(" {"),
    //                self.try_catch(
    //                    self.list([
    //                        self.text(" "),
    //                        parameter_list_doc.0,
    //                        self.text(" "),
    //                    ]),
    //                    parameter_list_doc.1,
    //                ),
    //                self.text("}"),
    //            ]);
    //            self.list(list)
    //        }
    //    }
    //}
    //
    //pub fn build_enum_variant(&self, variant: &EnumVariant) -> DocumentIdx {
    //    let mut list = vec![self.text(variant.0.to_string())];
    //    if let Some(parameter_list) = &variant.1 {
    //        let parameter_list_doc =
    // self.build_parameter_list(parameter_list);        list.push(
    //            self.try_catch(parameter_list_doc.0, parameter_list_doc.1),
    //        );
    //    }
    //    self.list(list)
    //}
    //
    //pub fn build_module(&self, item: &Loc<ast::Module>) -> DocumentIdx {
    //    self.list([
    //        self.text(format!("mod {} {{", item.name)),
    //        self.newline(),
    //        self.nest(self.build_module_body(&item.body), self.indent),
    //        self.newline(),
    //        self.text("}}"),
    //    ])
    //}
    //
    //pub fn build_module_body(
    //    &self,
    //    body: &Loc<ast::ModuleBody>,
    //) -> DocumentIdx {
    //    let mut list = vec![];
    //    for (i, item) in body.members.iter().enumerate() {
    //        if i > 0 {
    //            list.push(self.newline());
    //            list.push(self.newline());
    //        }
    //        list.push(self.build_item(item));
    //    }
    //    self.list(list)
    //}
    //
    //pub fn build_use(
    //    &self,
    //    use_statement: &Loc<ast::UseStatement>,
    //) -> DocumentIdx {
    //    let ast::UseStatement { path, alias } = &use_statement.inner;
    //
    //    let mut line = vec![self.text("use "), self.build_path(path)];
    //
    //    if let Some(alias) = alias {
    //        line.push(self.text(format!(" as {}", alias)));
    //    }
    //
    //    line.push(self.text(";"));
    //    self.list(line)
    //}
    //
    //pub fn build_impl_block(
    //    &self,
    //    impl_block: &Loc<ast::ImplBlock>,
    //) -> DocumentIdx {
    //    let mut list = vec![self.text("impl")];
    //    if let Some(type_params) = &impl_block.type_params {
    //        list.push(self.group(
    //            lexer::TokenKind::Lt.as_str(),
    //            &type_params.inner,
    //            lexer::TokenKind::Comma,
    //            lexer::TokenKind::Gt.as_str(),
    //        ));
    //    }
    //    list.push(self.text(" "));
    //    if let Some(impl_trait) = &impl_block.r#trait {
    //        list.extend([
    //            self.build_trait_spec(impl_trait),
    //            self.text(" for "),
    //        ]);
    //    }
    //    list.push(self.build_type_spec(&impl_block.target));
    //
    //    if !impl_block.where_clauses.is_empty() {
    //        todo!()
    //    }
    //
    //    list.push(self.text(" {"));
    //    if !impl_block.units.is_empty() {
    //        list.push(self.newline());
    //        let mut unit_list = vec![];
    //        for (i, unit) in impl_block.units.iter().enumerate() {
    //            if i > 0 {
    //                unit_list.push(self.newline());
    //            }
    //            unit_list.push(self.build_unit(unit))
    //        }
    //        list.push(self.nest(self.list(unit_list), self.indent));
    //        list.push(self.newline());
    //    }
    //    list.push(self.text("}"));
    //
    //    self.list(list)
    //}
    //
    //pub fn build_path(&self, path: &Loc<Path>) -> DocumentIdx {
    //    self.text(
    //        path.inner
    //            .0
    //            .iter()
    //            .map(|component| component.to_string())
    //            .collect::<Vec<_>>()
    //            .join("::"),
    //    )
    //}
    //
    //pub fn build_statement(
    //    &self,
    //    statement: &Loc<ast::Statement>,
    //) -> DocumentIdx {
    //    let mut list = match &**statement {
    //        ast::Statement::Label(loc) => todo!(),
    //        ast::Statement::Declaration(vec) => todo!(),
    //        ast::Statement::Binding(binding) => {
    //            let mut list = vec![
    //                self.text("let "),
    //                self.build_pattern(&binding.pattern),
    //            ];
    //
    //            if let Some(ty) = &binding.ty {
    //                list.extend([self.text(": "), self.build_type_spec(ty)]);
    //            }
    //
    //            list.push(self.text(" = "));
    //            list.push(self.build_expression(&binding.value));
    //
    //            list
    //        }
    //        ast::Statement::PipelineRegMarker(loc, loc1) => {
    //            todo!()
    //        }
    //        ast::Statement::Register(register) => {
    //            let mut list = vec![
    //                self.text("reg("),
    //                self.build_expression(&register.clock),
    //                self.text(") "),
    //                self.build_pattern(&register.pattern),
    //                self.text(" "),
    //            ];
    //
    //            if !register.attributes.0.is_empty()
    //                || register.value_type.is_some()
    //                || register.initial.is_some()
    //            {
    //                todo!()
    //            }
    //
    //            if let Some(reset) = &register.reset {
    //                list.extend([
    //                    self.text("reset("),
    //                    self.build_expression(&reset.0),
    //                    self.text(": "),
    //                    self.build_expression(&reset.1),
    //                    self.text(") "),
    //                ]);
    //            }
    //
    //            list.extend([
    //                self.text("= "),
    //                self.build_expression(&register.value),
    //            ]);
    //
    //            list
    //        }
    //        ast::Statement::Set { target, value } => vec![
    //            self.text("set "),
    //            self.build_expression(target),
    //            self.text(" = "),
    //            self.build_expression(value),
    //        ],
    //        ast::Statement::Assert(loc) => todo!(),
    //    };
    //    list.push(self.text(";"));
    //    self.list(list)
    //}
    //
    //pub fn build_expression(
    //    &self,
    //    expression: &Loc<ast::Expression>,
    //) -> DocumentIdx {
    //    match &**expression {
    //        ast::Expression::Identifier(path) => self.build_path(path),
    //        ast::Expression::IntLiteral(int_literal) => {
    //            self.text(int_literal.to_string())
    //        }
    //        ast::Expression::BoolLiteral(bool_literal) => {
    //            self.text(bool_literal.to_string())
    //        }
    //        ast::Expression::BitLiteral(bit_literal) => {
    //            self.text(match bit_literal {
    //                ast::BitLiteral::Low => "LOW",
    //                ast::BitLiteral::High => "HIGH",
    //                ast::BitLiteral::HighImp => "UNDEF",
    //            })
    //        }
    //        ast::Expression::ArrayLiteral(array_literal) => self.group(
    //            lexer::TokenKind::OpenBracket.as_str(),
    //            array_literal,
    //            lexer::TokenKind::Comma,
    //            lexer::TokenKind::CloseBracket.as_str(),
    //        ),
    //        ast::Expression::ArrayShorthandLiteral(loc, loc1) => todo!(),
    //        ast::Expression::Index(loc, loc1) => todo!(),
    //        ast::Expression::RangeIndex { target, start, end } => todo!(),
    //        ast::Expression::TupleLiteral(items) => self.group(
    //            lexer::TokenKind::OpenParen.as_str(),
    //            items,
    //            lexer::TokenKind::Comma,
    //            lexer::TokenKind::CloseParen.as_str(),
    //        ),
    //        ast::Expression::TupleIndex(loc, loc1) => todo!(),
    //        ast::Expression::FieldAccess(parent, field) => self.list([
    //            self.build_expression(parent),
    //            self.text(format!(".{}", field)),
    //        ]),
    //        ast::Expression::CreatePorts => todo!(),
    //        ast::Expression::Call {
    //            kind,
    //            callee,
    //            args,
    //            turbofish,
    //        } => {
    //            let mut list = match kind {
    //                ast::CallKind::Function => vec![],
    //                ast::CallKind::Entity(_) => vec![self.text("inst ")],
    //                ast::CallKind::Pipeline(_, latency) => vec![
    //                    self.text("inst("),
    //                    self.build_type_expression(latency),
    //                    self.text(") "),
    //                ],
    //            };
    //
    //            list.push(self.build_path(callee));
    //            if let Some(turbofish) = turbofish {
    //                list.push(self.build_turbofish(turbofish));
    //            }
    //            list.push(self.build_argument_list(args));
    //
    //            self.list(list)
    //        }
    //        ast::Expression::MethodCall {
    //            target,
    //            name,
    //            args,
    //            kind,
    //            turbofish,
    //        } => {
    //            let mut list = vec![
    //                self.text("("),
    //                self.build_expression(target),
    //                self.text(")."),
    //            ];
    //            list.extend(match kind {
    //                ast::CallKind::Function => vec![],
    //                ast::CallKind::Entity(_) => vec![self.text("inst ")],
    //                ast::CallKind::Pipeline(_, latency) => vec![
    //                    self.text("inst("),
    //                    self.build_type_expression(latency),
    //                    self.text(") "),
    //                ],
    //            });
    //
    //            list.push(self.text(name.to_string()));
    //
    //            if let Some(turbofish) = turbofish {
    //                list.push(self.build_turbofish(turbofish))
    //            }
    //
    //            list.push(self.build_argument_list(args));
    //
    //            self.list(list)
    //        }
    //        ast::Expression::If(condition, true_branch, false_branch) => self
    //            .list([
    //                self.text("if "),
    //                self.build_expression(condition),
    //                self.text(" "),
    //                self.build_expression(true_branch),
    //                self.text(" else "),
    //                self.build_expression(false_branch),
    //            ]),
    //        ast::Expression::Match(against, arms) => {
    //            let mut list =
    //                vec![self.text("match "), self.build_expression(against)];
    //            if !arms.is_empty() {
    //                let mut arm_list = vec![];
    //                for arm in &arms.inner {
    //                    let pattern = self.build_pattern(&arm.0);
    //                    let case = self.list([
    //                        self.text(format!(
    //                            " {} ",
    //                            lexer::TokenKind::FatArrow.as_str()
    //                        )),
    //                        self.build_expression(&arm.1),
    //                    ]);
    //                    arm_list.push(self.try_catch(
    //                        self.list([
    //                            self.flatten(pattern),
    //                            self.flatten(case),
    //                        ]),
    //                        self.try_catch(
    //                            self.list([self.flatten(pattern), case]),
    //                            self.list([pattern, case]),
    //                        ),
    //                    ));
    //                }
    //
    //                let arms_doc =
    //                    self.group_raw(&arm_list, lexer::TokenKind::Comma);
    //                list.extend([
    //                    self.text(" {"),
    //                    self.try_catch(
    //                        self.list([
    //                            self.text(" "),
    //                            arms_doc.0,
    //                            self.text(" "),
    //                        ]),
    //                        arms_doc.1,
    //                    ),
    //                    self.text("}"),
    //                ]);
    //            }
    //            self.list(list)
    //        }
    //        // TODO: proper parenthesization in both of these
    //        ast::Expression::UnaryOperator(unary_operator, inner) => {
    //            self.list([
    //                self.text(unary_operator.to_string()),
    //                self.build_expression(inner),
    //            ])
    //        }
    //        ast::Expression::BinaryOperator(left, op, right) => self.list([
    //            self.build_expression(left),
    //            self.text(format!(" {} ", op)),
    //            self.build_expression(right),
    //        ]),
    //        ast::Expression::Block(block) => {
    //            let mut list = vec![self.token(lexer::TokenKind::OpenBrace)];
    //            if block.statements.len()
    //                + block.result.as_ref().map_or(0, |_| 1)
    //                > 0
    //            {
    //                list.push(self.newline());
    //
    //                let mut nest = vec![];
    //
    //                for statement in &block.statements {
    //                    nest.push(self.build_statement(statement));
    //                    nest.push(self.newline());
    //                }
    //
    //                if let Some(result) = &block.result {
    //                    nest.push(self.build_expression(result));
    //                    nest.push(self.newline());
    //                }
    //
    //                list.push(self.nest(self.list(nest), self.indent));
    //            }
    //            list.push(self.token(lexer::TokenKind::CloseBrace));
    //
    //            self.list(list)
    //        }
    //        ast::Expression::PipelineReference {
    //            stage_kw_and_reference_loc,
    //            stage,
    //            name,
    //        } => todo!(),
    //        ast::Expression::TypeLevelIf(loc, loc1, loc2) => todo!(),
    //        ast::Expression::StageValid => todo!(),
    //        ast::Expression::StageReady => todo!(),
    //    }
    //}
    //
    //pub fn build_turbofish(
    //    &self,
    //    turbofish: &Loc<ast::TurbofishInner>,
    //) -> DocumentIdx {
    //    match &**turbofish {
    //        ast::TurbofishInner::Named(vec) => todo!(),
    //        ast::TurbofishInner::Positional(arguments) => self.list([
    //            self.text("::"),
    //            self.group(
    //                lexer::TokenKind::Lt.as_str(),
    //                arguments,
    //                lexer::TokenKind::Comma,
    //                lexer::TokenKind::Gt.as_str(),
    //            ),
    //        ]),
    //    }
    //}
    //
    //pub fn build_argument_list(
    //    &self,
    //    argument_list: &Loc<ast::ArgumentList>,
    //) -> DocumentIdx {
    //    match &**argument_list {
    //        ast::ArgumentList::Positional(arguments) => self.group(
    //            lexer::TokenKind::OpenParen.as_str(),
    //            arguments,
    //            lexer::TokenKind::Comma,
    //            lexer::TokenKind::CloseParen.as_str(),
    //        ),
    //        ast::ArgumentList::Named(named_arguments) => self.group(
    //            "$(",
    //            named_arguments,
    //            lexer::TokenKind::Comma,
    //            lexer::TokenKind::CloseParen.as_str(),
    //        ),
    //    }
    //}
    //
    //pub fn build_named_argument(
    //    &self,
    //    named_argument: &ast::NamedArgument,
    //) -> DocumentIdx {
    //    match named_argument {
    //        ast::NamedArgument::Full(name, current) => self.list([
    //            self.text(format!("{}: ", name)),
    //            self.build_expression(current),
    //        ]),
    //        ast::NamedArgument::Short(name) => self.text(name.to_string()),
    //    }
    //}
    //
    //pub fn build_pattern(&self, pattern: &Loc<ast::Pattern>) -> DocumentIdx {
    //    match &**pattern {
    //        ast::Pattern::Integer(int_literal) => {
    //            self.text(int_literal.to_string())
    //        }
    //        ast::Pattern::Bool(bool_literal) => {
    //            self.text(bool_literal.to_string())
    //        }
    //        ast::Pattern::Path(path) => self.build_path(path),
    //        ast::Pattern::Tuple(tuple) => self.group(
    //            lexer::TokenKind::OpenParen.as_str(),
    //            tuple,
    //            lexer::TokenKind::Comma,
    //            lexer::TokenKind::CloseParen.as_str(),
    //        ),
    //        ast::Pattern::Array(vec) => todo!(),
    //        ast::Pattern::Type(name, argument_pattern) => self.list([
    //            self.build_path(name),
    //            self.build_argument_pattern(argument_pattern),
    //        ]),
    //    }
    //}
    //
    //pub fn build_argument_pattern(
    //    &self,
    //    argument_pattern: &Loc<ast::ArgumentPattern>,
    //) -> DocumentIdx {
    //    match &**argument_pattern {
    //        ast::ArgumentPattern::Named(vec) => todo!(),
    //        ast::ArgumentPattern::Positional(tuple) => self.group(
    //            lexer::TokenKind::OpenParen.as_str(),
    //            tuple,
    //            lexer::TokenKind::Comma,
    //            lexer::TokenKind::CloseParen.as_str(),
    //        ),
    //    }
    //}
    //
    //pub fn build_type_expression(
    //    &self,
    //    type_expression: &Loc<ast::TypeExpression>,
    //) -> DocumentIdx {
    //    match &**type_expression {
    //        ast::TypeExpression::TypeSpec(type_spec) => {
    //            self.build_type_spec(type_spec)
    //        }
    //        ast::TypeExpression::Integer(value) =>
    // self.text(value.to_string()),
    //        ast::TypeExpression::ConstGeneric(expression) => {
    //            self.build_expression(expression)
    //        }
    //    }
    //}
    //
    //pub fn build_type_spec(
    //    &self,
    //    type_spec: &Loc<ast::TypeSpec>,
    //) -> DocumentIdx {
    //    match &**type_spec {
    //        ast::TypeSpec::Tuple(elements) => self.group(
    //            lexer::TokenKind::OpenParen.as_str(),
    //            elements,
    //            lexer::TokenKind::Comma,
    //            lexer::TokenKind::CloseParen.as_str(),
    //        ),
    //        ast::TypeSpec::Array { inner, size } => self.list([
    //            self.text("["),
    //            self.build_type_expression(inner),
    //            self.text("; "),
    //            self.build_type_expression(size),
    //            self.text("]"),
    //        ]),
    //        ast::TypeSpec::Named(path, type_params) => {
    //            let mut list = vec![self.build_path(path)];
    //            if let Some(params) = type_params {
    //                list.push(self.group(
    //                    lexer::TokenKind::Lt.as_str(),
    //                    &params.inner,
    //                    lexer::TokenKind::Comma,
    //                    lexer::TokenKind::Gt.as_str(),
    //                ));
    //            }
    //            self.list(list)
    //        }
    //        ast::TypeSpec::Inverted(inner) => self
    //            .list([self.text("inv "), self.build_type_expression(inner)]),
    //        ast::TypeSpec::Wire(inner) => {
    //            self.list([self.text("&"), self.build_type_expression(inner)])
    //        }
    //        ast::TypeSpec::Wildcard => self.text("_"),
    //    }
    //}
    //
    //pub fn build_type_param(
    //    &self,
    //    type_param: &Loc<ast::TypeParam>,
    //) -> DocumentIdx {
    //    match &**type_param {
    //        ast::TypeParam::TypeName { name, traits } => {
    //            let mut list = vec![self.text(name.to_string())];
    //            if !traits.is_empty() {
    //                let mut flatten_list = vec![];
    //                let mut nest_list = vec![];
    //                for (i, trait_spec) in traits.iter().enumerate() {
    //                    if i > 0 {
    //                        flatten_list.push(self.text(format!(
    //                            " {} ",
    //                            lexer::TokenKind::Plus.as_str()
    //                        )));
    //                        nest_list.extend([
    //                            self.newline(),
    //                            self.text(format!(
    //                                "{} ",
    //                                lexer::TokenKind::Plus.as_str()
    //                            )),
    //                        ])
    //                    }
    //                    flatten_list.push(self.build_trait_spec(trait_spec));
    //                    nest_list.push(self.build_trait_spec(trait_spec));
    //                }
    //                list.extend([
    //                    self.text(": "),
    //                    self.try_catch(
    //                        self.flatten(self.list(flatten_list)),
    //                        self.nest(self.list(nest_list), self.indent),
    //                    ),
    //                ])
    //            }
    //            self.list(list)
    //        }
    //        ast::TypeParam::TypeWithMeta { meta, name } => {
    //            self.text(format!("#{} {}", meta, name))
    //        }
    //    }
    //}
    //
    //pub fn build_trait_spec(
    //    &self,
    //    trait_spec: &Loc<ast::TraitSpec>,
    //) -> DocumentIdx {
    //    let mut list = vec![self.build_path(&trait_spec.path)];
    //    if let Some(type_params) = &trait_spec.type_params {
    //        list.push(self.group(
    //            lexer::TokenKind::Lt.as_str(),
    //            &type_params.inner,
    //            lexer::TokenKind::Comma,
    //            lexer::TokenKind::Gt.as_str(),
    //        ));
    //    }
    //    self.list(list)
    //}
    //
    //pub fn build_attribute(
    //    &self,
    //    attribute: &Loc<ast::Attribute>,
    //) -> DocumentIdx {
    //    match &**attribute {
    //        ast::Attribute::Optimize { passes } => todo!(),
    //        ast::Attribute::NoMangle { all } => self.text(format!(
    //            "#[no_mangle{}]",
    //            if *all { "(all)" } else { "" }
    //        )),
    //        ast::Attribute::Fsm { state } => todo!(),
    //        ast::Attribute::WalTraceable {
    //            suffix,
    //            uses_clk,
    //            uses_rst,
    //        } => todo!(),
    //        ast::Attribute::WalTrace { clk, rst } => todo!(),
    //        ast::Attribute::WalSuffix { suffix } => todo!(),
    //    }
    //}
    //
    //pub fn build_attribute_list(
    //    &self,
    //    attribute_list: &ast::AttributeList,
    //    always_newline: bool,
    //) -> DocumentIdx {
    //    self.list(match attribute_list.0.len() {
    //        0 => vec![],
    //        1 => vec![
    //            self.build_attribute(&attribute_list.0[0]),
    //            if always_newline {
    //                self.newline()
    //            } else {
    //                self.text(" ")
    //            },
    //        ],
    //        _ => {
    //            let mut list = vec![];
    //            for attribute in &attribute_list.0 {
    //                list.extend([
    //                    self.build_attribute(attribute),
    //                    self.newline(),
    //                ]);
    //            }
    //            list
    //        }
    //    })
    //}
    //
    //pub fn build_parameter(&self, parameter: &AstParameter) -> DocumentIdx {
    //    self.list([
    //        self.build_attribute_list(&parameter.0, false),
    //        self.text(format!("{}: ", parameter.1)),
    //        self.build_type_spec(&parameter.2),
    //    ])
    //}
    //
    //pub fn build_parameter_list(
    //    &self,
    //    parameter_list: &Loc<ast::ParameterList>,
    //) -> (DocumentIdx, DocumentIdx) {
    //    let mut try_list = vec![];
    //    let mut catch_list = vec![];
    //    if parameter_list.self_.is_some() {
    //        let continues = !parameter_list.args.is_empty();
    //        try_list.push(self.text(if continues { "self, " } else { "self"
    // }));        catch_list.extend([
    //            self.newline(),
    //            self.nest(self.text("self,"), self.indent),
    //        ]);
    //    }
    //    let (try_idx, catch_idx) =
    //        self.group_raw(&parameter_list.args, lexer::TokenKind::Comma);
    //    try_list.push(try_idx);
    //    catch_list.push(catch_idx);
    //    (self.list(try_list), self.list(catch_list))
    //}

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
        &self,
        try_body: DocumentIdx,
        catch_body: DocumentIdx,
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

    fn group_raw<'a, B: BuildAsDocument + 'a>(
        &self,
        contents: impl IntoIterator<Item = &'a B>,
        between: impl Into<Option<lexer::TokenKind>>,
    ) -> (DocumentIdx, DocumentIdx) {
        let between = between.into();

        let mut list = vec![];
        for (i, item) in contents
            .into_iter()
            .map(|item| item.build(self))
            .enumerate()
        {
            if i > 0 {
                if let Some(ref between) = between {
                    list.extend([self.token(between.clone()), self.newline()]);
                }
            }
            list.push(item);
        }
        let doc_contents = self.list(list);
        let mut nest_list =
            vec![self.newline(), self.nest(doc_contents, self.indent)];
        if matches!(between, Some(lexer::TokenKind::Comma)) {
            // always trailing comma when nesting a comma group, could
            // overestimate
            nest_list.push(self.token(lexer::TokenKind::Comma));
        }
        nest_list.push(self.newline());
        // try to flatten, otherwise nest
        (self.flatten(doc_contents), self.list(nest_list))
    }

    fn group<'a, B: BuildAsDocument + 'a>(
        &self,
        open: impl Into<String>,
        contents: impl IntoIterator<Item = &'a B>,
        between: impl Into<Option<lexer::TokenKind>>,
        close: impl Into<String>,
    ) -> DocumentIdx {
        let open = open.into();
        let close = close.into();

        let (try_body_idx, catch_body_idx) = self.group_raw(contents, between);
        let mut try_list = vec![];
        let mut catch_list = vec![];
        //if let Some(open) = open {
        try_list.push(self.text(open.clone()));
        catch_list.push(self.text(open));
        //}
        try_list.push(try_body_idx);
        catch_list.push(catch_body_idx);
        //if let Some(close) = close {
        try_list.push(self.text(close.clone()));
        catch_list.push(self.text(close));
        //}
        self.try_catch(self.list(try_list), self.list(catch_list))
    }
}
