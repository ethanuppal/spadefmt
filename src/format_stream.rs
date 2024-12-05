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

use core::panic;
use std::fmt;

#[derive(Clone, Copy)]
pub enum HighlightGroup {
    None,
    Identifier,
    Keyword,
    NonterminalPathSegment,
    TerminalPathSegment,
    TypeName,
    Literal,
    Symbol,
    Attribute,
}

pub trait FormatStream {
    fn indent(&mut self) -> fmt::Result;

    fn dedent(&mut self) -> fmt::Result;

    fn newline(&mut self) -> fmt::Result;

    /// `code` is guaranteed to contain no newlines.
    fn process_code(
        &mut self, code: &str, highlight_group: HighlightGroup,
    ) -> fmt::Result;

    fn identifier(&mut self, name: &str) -> fmt::Result {
        self.process_code(name, HighlightGroup::Identifier)
    }

    fn keyword(&mut self, keyword: &str) -> fmt::Result {
        self.process_code(keyword, HighlightGroup::Keyword)
    }

    fn nonterminal_segment(&mut self, segment: &str) -> fmt::Result {
        self.process_code(segment, HighlightGroup::NonterminalPathSegment)
    }

    fn terminal_segment(&mut self, segment: &str) -> fmt::Result {
        self.process_code(segment, HighlightGroup::TerminalPathSegment)
    }

    fn type_name(&mut self, name: &str) -> fmt::Result {
        self.process_code(name, HighlightGroup::TypeName)
    }

    fn literal(&mut self, literal: &str) -> fmt::Result {
        self.process_code(literal, HighlightGroup::Literal)
    }

    fn symbol(&mut self, symbol: &str) -> fmt::Result {
        self.process_code(symbol, HighlightGroup::Literal)
    }

    fn attribute(&mut self, attribute: &str) -> fmt::Result {
        self.process_code("#", HighlightGroup::Attribute)?;
        self.symbol("[")?;
        self.process_code(attribute, HighlightGroup::Attribute)?;
        self.symbol("]")
    }

    fn space(&mut self) -> fmt::Result {
        self.process_code(" ", HighlightGroup::None)
    }
}

///  `ConcreteMark { line, column, indent_level }` means there is a mark set at
/// line `line` and column `column`, both zero-indexed, where the indent level
/// was `indent_level`.
struct ConcreteMark {
    line: usize,
    column: usize,
    indent_level: usize,
    should_commit: bool,
    width_limit_failure: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Mark(usize);

pub struct CommitableFormatStream<'stream> {
    inner: &'stream mut dyn FormatStream,
    indent: usize,
    indent_level: usize,
    just_got_newline: bool,
    should_commit: bool,
    max_width: usize,
    /// Whether the column limit of `self.max_width` has been exceeded *while
    /// non-comittable*.
    width_limit_failure: bool,
    /// Invariant: `!lines.is_empty() && levels.len() == lines.len()`.
    lines: Vec<usize>,
    marks: Vec<ConcreteMark>,
}

impl<'stream> CommitableFormatStream<'stream> {
    pub fn new_with_config(
        inner: &'stream mut dyn FormatStream, indent: usize, max_width: usize,
    ) -> Self {
        Self {
            inner,
            indent,
            just_got_newline: false,
            should_commit: true,
            max_width,
            width_limit_failure: false,
            indent_level: 0,
            lines: vec![1],
            marks: Vec::new(),
        }
    }

    pub fn current_width(&self) -> usize {
        let last_line = self.lines.len() - 1;
        self.lines[last_line]
    }

    /// Sets a new location mark, saving state like whether the stream is
    /// comitting or whether theere has been a width error.
    pub fn push_mark(&mut self) -> Mark {
        let current_line = self.lines.len() - 1;
        self.marks.push(ConcreteMark {
            line: current_line,
            column: self.lines[current_line],
            indent_level: self.indent_level,
            should_commit: self.should_commit,
            width_limit_failure: self.width_limit_failure,
        });
        Mark(self.marks.len() - 1)
    }

    /// Restores the given mark, consuming it and any mark set after it. Panics
    /// if the mark has not been set.
    pub fn return_to_mark(&mut self, mark: Mark) {
        let Mark(mark_index) = mark;

        if mark_index >= self.marks.len() {
            panic!("no mark set");
        }

        let ConcreteMark {
            line,
            column,
            indent_level,
            should_commit,
            width_limit_failure,
        } = self.marks[mark_index];
        self.marks.truncate(mark_index);

        self.lines.truncate(line + 1);
        self.lines[line] = column;
        self.indent_level = indent_level;
        self.should_commit = should_commit;
        self.width_limit_failure = width_limit_failure;
    }

    #[inline]
    pub fn enable_commit(&mut self) {
        self.should_commit = true;
        self.width_limit_failure = false;
    }

    #[inline]
    pub fn disable_commit(&mut self) {
        self.should_commit = false;
    }

    #[inline]
    pub fn width_limit_failure(&self) -> bool {
        self.width_limit_failure
    }
}

impl FormatStream for CommitableFormatStream<'_> {
    fn indent(&mut self) -> fmt::Result {
        self.indent_level += self.indent;

        if self.should_commit {
            self.inner.indent()?;
        } else if self.current_width() > self.max_width {
            self.width_limit_failure = true;
        }

        Ok(())
    }

    fn dedent(&mut self) -> fmt::Result {
        self.indent_level -= self.indent;

        if self.should_commit {
            self.inner.dedent()?;
        }

        Ok(())
    }

    fn newline(&mut self) -> fmt::Result {
        self.just_got_newline = true;
        self.lines.push(0);

        if self.should_commit {
            self.inner.newline()?;
        }

        Ok(())
    }

    fn process_code(
        &mut self, code: &str, highlight_group: HighlightGroup,
    ) -> fmt::Result {
        let last_line = self.lines.len() - 1;
        if self.just_got_newline {
            self.lines[last_line] += self.indent_level;
            self.just_got_newline = false;
        }
        self.lines[last_line] += code.len();

        if self.should_commit {
            self.inner.process_code(code, highlight_group)?;
        } else if self.current_width() > self.max_width {
            self.width_limit_failure = true;
        }

        Ok(())
    }
}
