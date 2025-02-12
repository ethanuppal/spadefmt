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

use crate::document::{Document, DocumentIdx, InternedDocumentStore};

#[derive(Default, Clone, Debug)]
pub struct PrintingContext {
    max_width: usize,
    column: usize,
    current_indent: usize,
    flatten: bool,
    trying: bool,
    tainted: bool,
}

impl PrintingContext {
    pub fn new(max_width: usize) -> Self {
        Self {
            max_width,
            ..Default::default()
        }
    }

    fn newline(&mut self) {
        if self.flatten {
            self.column += 1;
        } else {
            self.column = self.current_indent;
        }
        if self.column > self.max_width {
            //println!("oops tainted {:?}", self);
            self.tainted = true;
        }
    }

    fn indent(&mut self, by: isize) {
        self.current_indent = (self.current_indent as isize + by) as usize;
    }

    fn push(&mut self, length: usize) {
        self.column += length;
        if self.column > self.max_width {
            self.tainted = true;
        }
    }

    fn set_flattened(&mut self) {
        self.flatten = true;
    }
}

// TODO: maybe merge top function into this
/// Invariant: A try will never be expanded after a catch.
pub fn resolve_try_catch(
    store: &mut InternedDocumentStore, idx: DocumentIdx,
    context: &mut PrintingContext,
) -> DocumentIdx {
    match store.get(idx).clone() {
        Document::Newline => {
            context.newline();
            idx
        }
        Document::Text(text) => {
            context.push(text.len());
            idx
        }
        Document::Nest(body_idx, by) => {
            context.indent(by);
            let new_body_idx = resolve_try_catch(store, body_idx, context);
            context.indent(-by);
            store.add(Document::Nest(new_body_idx, by))
        }
        Document::Flatten(body_idx) => {
            let mut flattened_context = context.clone();
            flattened_context.set_flattened();
            let new_body_idx =
                resolve_try_catch(store, body_idx, &mut flattened_context);
            flattened_context.flatten = context.flatten;
            *context = flattened_context;
            store.add(Document::Flatten(new_body_idx))
        }
        Document::List(children) => {
            let new_children = children
                .into_iter()
                .map(|child_idx| resolve_try_catch(store, child_idx, context))
                .collect();
            store.add(Document::List(new_children))
        }
        Document::TryCatch(try_body_idx, catch_body_idx) => {
            let mut try_context = context.clone();
            try_context.trying = true;

            //println!("\ntrying from {:?}", try_context);
            //let mut buffer = String::new();
            //let mut f = inform::fmt::IndentWriter::new(&mut buffer, 4);
            //crate::document::debug_print(store, &mut f, try_body_idx)
            //    .expect("a");
            //println!("{}", buffer);

            let new_try_body_idx =
                resolve_try_catch(store, try_body_idx, &mut try_context);
            if try_context.tainted && !context.trying {
                let mut catch_context = context.clone();

                //println!(
                //    "\nfailed to flatten, doing nest from {:?}",
                //    catch_context
                //);
                //let mut buffer = String::new();
                //let mut f = inform::fmt::IndentWriter::new(&mut buffer, 4);
                //crate::document::debug_print(store, &mut f, catch_body_idx)
                //    .expect("a");
                //println!("{}", buffer);

                let new_catch_body_idx = resolve_try_catch(
                    store,
                    catch_body_idx,
                    &mut catch_context,
                );
                *context = catch_context;
                new_catch_body_idx
            } else {
                try_context.trying = context.trying;
                *context = try_context;
                new_try_body_idx
            }
        }
    }
}
