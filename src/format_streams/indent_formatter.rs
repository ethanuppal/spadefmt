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

use std::{fmt, io::Write};

use codespan_reporting::term::termcolor::{Buffer, WriteColor};
use inform::{common::IndentWriterCommon, io::IndentWriter};

use crate::format_stream::{FormatStream, HighlightGroup};

use super::Theme;

pub struct IndentFormatterStream<'buffer> {
    theme: Theme,
    f: IndentWriter<'buffer, Buffer>,
}

impl<'buffer> IndentFormatterStream<'buffer> {
    pub fn new(theme: Theme, f: IndentWriter<'buffer, Buffer>) -> Self {
        Self { theme, f }
    }
}

impl FormatStream for IndentFormatterStream<'_> {
    fn indent(&mut self) -> fmt::Result {
        self.f.increase_indent();
        Ok(())
    }

    fn dedent(&mut self) -> fmt::Result {
        self.f.decrease_indent();
        Ok(())
    }

    fn newline(&mut self) -> fmt::Result {
        writeln!(self.f).map_err(|_| fmt::Error)
    }

    fn process_code(
        &mut self, code: &str, highlight_group: HighlightGroup,
    ) -> fmt::Result {
        self.f.indent_if_needed();
        let color = self.theme.color_for(code, highlight_group);
        self.f.with_raw_buffer(|buffer| {
            buffer.set_color(color).map_err(|_| fmt::Error)?;
            write!(buffer, "{}", code).map_err(|_| fmt::Error)
        })
    }
}
