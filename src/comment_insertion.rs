// Copyright (C) 2025 Ethan Uppal.
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

use std::{collections::VecDeque, fmt, fmt::Write};

use spade_codespan_reporting::files::{Files, SimpleFile};
use spade_parser::Comment;

use crate::document::ResolvedPrintingContext;

pub struct CommentToPrint<'parser, 'source> {
    pub inner: &'parser Comment,
    pub source: &'source str,
}

impl CommentToPrint<'_, '_> {
    fn start_line(&self, file: &SimpleFile<String, String>) -> usize {
        file.line_index(
            (),
            match self.inner {
                Comment::Line(token) | Comment::Block(token, ..) => {
                    token.span.start
                }
            },
        )
        .unwrap()
    }
}

pub struct CommentInserter<'parser, 'source> {
    comments: VecDeque<CommentToPrint<'parser, 'source>>,
}

impl<'parser, 'source> CommentInserter<'parser, 'source> {
    pub fn new(comments: &'parser [Comment], source: &'source str) -> Self {
        Self {
            comments: comments
                .iter()
                .inspect(|comment| {
                    println!("{comment:?}");
                })
                .map(|comment| CommentToPrint {
                    inner: comment,
                    source: match comment {
                        Comment::Line(token) => &source[token.span.clone()],
                        Comment::Block(start_token, end_token) => {
                            &source[start_token.span.start..end_token.span.end]
                        }
                    },
                })
                .collect(),
        }
    }

    // pub fn get_comment(
    //     &mut self,
    //     context: &ResolvedPrintingContext,
    // ) -> Option<CommentToPrint<'parser, 'source>> {
    //     if let Some(first) = self.comments.front() && context.line ==
    // first.start_line {         self.comments.pop_front()
    //     } else {
    //         None
    //     }
    // }

    pub fn get_comments_temp(
        &mut self,
        file: &SimpleFile<String, String>,
        start_line_index: usize,
        end_line_index: usize,
    ) -> Vec<CommentToPrint<'parser, 'source>> {
        let mut result = vec![];
        // commented out because this way we guarantee we don't lose any
        // comments while let Some(comment) = self.comments.pop_front()
        // {     if comment.start_line(file) >= start_line_index {
        //         self.comments.push_front(comment);
        //         break;
        //     }
        // }
        while let Some(comment) = self.comments.pop_front() {
            if comment.start_line(file) >= end_line_index {
                self.comments.push_front(comment);
                break;
            }
            result.push(comment);
        }
        result
    }
}

pub fn print_comment_as_block<W: fmt::Write>(
    f: &mut inform::fmt::IndentWriter<W>,
    context: &mut ResolvedPrintingContext,
    comment: CommentToPrint,
) -> fmt::Result {
    match comment.inner {
        Comment::Line(..) => write!(f, "/* {} */", comment.source),
        Comment::Block(..) => print_comment_as_original(f, context, comment),
    }
}

pub fn print_comment_as_original<W: fmt::Write>(
    f: &mut inform::fmt::IndentWriter<W>,
    context: &mut ResolvedPrintingContext,
    comment: CommentToPrint,
) -> fmt::Result {
    match comment.inner {
        Comment::Line(..) => write!(f, "// {}", comment.source),
        Comment::Block(..) => {
            context.advance_lines(
                comment.source.chars().filter(|c| *c == '\n').count(),
            );
            write!(f, "/* {} */", comment.source)
        }
    }
}
