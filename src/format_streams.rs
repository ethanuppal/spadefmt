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

use codespan_reporting::term::termcolor::{Color, ColorSpec};

use crate::format_stream::HighlightGroup;

pub mod indent_formatter;

#[derive(Default)]
pub struct ColorSpecBuilder {
    spec: ColorSpec,
}

impl ColorSpecBuilder {
    pub fn fg(mut self, color: Color) -> Self {
        self.spec.set_fg(Some(color));
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.spec.set_bg(Some(color));
        self
    }

    pub fn bold(mut self) -> Self {
        self.spec.set_bold(true);
        self
    }

    pub fn intense(mut self) -> Self {
        self.spec.set_intense(true);
        self
    }

    pub fn underline(mut self) -> Self {
        self.spec.set_underline(true);
        self
    }

    pub fn dimmed(mut self) -> Self {
        self.spec.set_dimmed(true);
        self
    }

    pub fn italic(mut self) -> Self {
        self.spec.set_italic(true);
        self
    }

    pub fn strikethrough(mut self) -> Self {
        self.spec.set_strikethrough(true);
        self
    }

    pub fn reset(mut self) -> Self {
        self.spec.set_reset(true);
        self
    }

    pub fn build(self) -> ColorSpec {
        self.spec
    }
}

pub struct Theme {
    reset: ColorSpec,
    identifier: ColorSpec,
    keyword: ColorSpec,
    self_related: ColorSpec,
    nonterminal_path_segment: ColorSpec,
    terminal_path_segment: ColorSpec,
    type_name: ColorSpec,
    symbol: ColorSpec,
    literal: ColorSpec,
    attribute: ColorSpec,
}

impl Theme {
    pub fn idk() -> Self {
        Self {
            reset: ColorSpec::default(),
            identifier: ColorSpecBuilder::default()
                .fg(Color::Blue)
                .intense()
                .build(),
            keyword: ColorSpecBuilder::default()
                .fg(Color::Magenta)
                .italic()
                .build(),
            self_related: ColorSpecBuilder::default().fg(Color::Red).build(),
            nonterminal_path_segment: ColorSpecBuilder::default()
                .fg(Color::Cyan)
                .build(),
            terminal_path_segment: ColorSpecBuilder::default()
                .fg(Color::Cyan)
                .build(),
            type_name: ColorSpecBuilder::default()
                .fg(Color::Cyan)
                .intense()
                .build(),
            symbol: ColorSpecBuilder::default().fg(Color::White).build(),
            literal: ColorSpecBuilder::default()
                .fg(Color::Green)
                .intense()
                .build(),
            attribute: ColorSpecBuilder::default().fg(Color::Yellow).build(),
        }

        // pub fn with_background(color: Color) -> Self {
        //
        // }
    }

    pub fn color_for(
        &self,
        code: &str,
        highlight_group: HighlightGroup,
    ) -> &ColorSpec {
        match highlight_group {
            HighlightGroup::None => &self.reset,
            HighlightGroup::Identifier => &self.identifier,
            HighlightGroup::Keyword => {
                if code == "self" {
                    &self.self_related
                } else {
                    &self.keyword
                }
            }
            HighlightGroup::NonterminalPathSegment => {
                &self.nonterminal_path_segment
            }
            HighlightGroup::TerminalPathSegment => &self.terminal_path_segment,
            HighlightGroup::TypeName => {
                if code == "Self" {
                    &self.self_related
                } else {
                    &self.type_name
                }
            }
            HighlightGroup::Literal => &self.literal,
            HighlightGroup::Symbol => &self.symbol,
            HighlightGroup::Attribute => &self.attribute,
        }
    }
}
