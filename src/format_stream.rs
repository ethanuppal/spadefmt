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

    fn space(&mut self) -> fmt::Result {
        self.process_code(" ", HighlightGroup::None)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CommitState {
    Idle,
    Commit,
    Pause,
}

pub struct CommitableFormatStream<'stream> {
    inner: &'stream mut dyn FormatStream,
    indent: usize,
    indent_level: usize,
    just_got_newline: bool,
    state: CommitState,
    /// Invariant: `!lines.is_empty() && levels.len() == lines.len()`.
    lines: Vec<usize>,
    /// If `mark` is `Some((line, column, indent_level))`, there is a mark set
    /// at line `line` and column `column`, both zero-indexed, where the
    /// indent level was `indent_level`.
    mark: Option<(usize, usize, usize)>,
}

impl<'stream> CommitableFormatStream<'stream> {
    pub fn indenting_by(
        indent: usize, inner: &'stream mut dyn FormatStream,
    ) -> Self {
        Self {
            inner,
            indent,
            just_got_newline: false,
            state: CommitState::Idle,
            indent_level: 0,
            lines: vec![1],
            mark: None,
        }
    }

    pub fn current_width(&self) -> usize {
        let last_line = self.lines.len() - 1;
        self.lines[last_line]
    }

    pub fn set_mark(&mut self) {
        let current_line = self.lines.len() - 1;
        self.mark =
            Some((current_line, self.lines[current_line], self.indent_level))
    }

    pub fn return_to_mark(&mut self) {
        let (line, column, indent_level) = self.mark.expect("no mark set");
        self.lines.truncate(line + 1);
        self.lines[line] = column;
        self.indent_level = indent_level;
        self.mark = None;
    }

    pub fn commit<R, F: FnOnce(&mut Self) -> R>(&mut self, f: F) -> R {
        let before = self.state;
        self.state = CommitState::Commit;
        let result = f(self);
        self.state = before;
        result
    }

    pub fn pause<R, F: FnOnce(&mut Self) -> R>(&mut self, f: F) -> R {
        if self.state == CommitState::Idle {
            f(self)
        } else {
            self.state = CommitState::Pause;
            let result = f(self);
            self.state = CommitState::Idle;
            result
        }
    }

    pub fn enable_commit(&mut self) {
        self.state = CommitState::Commit;
    }

    pub fn disable_commit(&mut self) {
        self.state = CommitState::Idle;
    }
}

impl<'stream> FormatStream for CommitableFormatStream<'stream> {
    fn indent(&mut self) -> fmt::Result {
        self.indent_level += self.indent;

        if self.state == CommitState::Commit {
            self.inner.indent()?;
        }

        Ok(())
    }

    fn dedent(&mut self) -> fmt::Result {
        self.indent_level -= self.indent;

        if self.state == CommitState::Commit {
            self.inner.indent()?;
        }

        Ok(())
    }

    fn newline(&mut self) -> fmt::Result {
        self.just_got_newline = true;
        self.lines.push(0);

        if self.state == CommitState::Commit {
            self.inner.indent()?;
        }

        Ok(())
    }

    fn process_code(
        &mut self, code: &str, highlight_group: HighlightGroup,
    ) -> fmt::Result {
        if self.just_got_newline {
            self.lines.push(self.indent_level);
            self.just_got_newline = false;
        }
        let last_line = self.lines.len() - 1;
        self.lines[last_line] += code.len();

        if self.state == CommitState::Commit {
            self.inner.process_code(code, highlight_group)?;
        }

        Ok(())
    }
}
