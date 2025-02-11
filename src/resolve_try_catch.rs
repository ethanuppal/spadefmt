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

#[derive(Clone, Copy)]
struct PrintingContext {
    column: usize,
    current_indent: usize,
    flatten: bool,
}

fn fake_evaluate(
    store: &InternedDocumentStore, max_width: usize,
    context: &mut PrintingContext, idx: DocumentIdx,
) -> Result<(), ()> {
    match store.get(idx) {
        Document::Newline => {
            if context.flatten {
                context.column += 1;
            } else {
                context.column = context.current_indent;
            }
        }
        Document::Text(text) => {
            context.column += text.len();
            if context.column > max_width {
                return Err(());
            }
        }
        Document::Nest(body_idx, by) => {
            let mut nested_context = *context;
            nested_context.current_indent =
                (nested_context.current_indent as isize + *by) as usize;
            fake_evaluate(store, max_width, &mut nested_context, *body_idx)?;
            nested_context.current_indent = context.current_indent;
            *context = nested_context;
        }
        Document::Flatten(document_idx) => {}
        Document::List(vec) => todo!(),
        Document::TryCatch(try_body_idx, catch_body_idx) => {
            let mut try_context = *context;
            if let Err(()) =
                fake_evaluate(store, max_width, &mut try_context, *try_body_idx)
            {
                let mut catch_context = *context;
                fake_evaluate(
                    store,
                    max_width,
                    &mut catch_context,
                    *catch_body_idx,
                );
                *context = catch_context;
            } else {
                *context = try_context;
            }
        }
    }
    Ok(())
}

// TODO: maybe merge top function into this
pub fn resolve_try_catch(
    store: &mut InternedDocumentStore, idx: DocumentIdx,
) -> DocumentIdx {
    match store.get(idx).clone() {
        Document::Newline => idx,
        Document::Text(_) => idx,
        Document::Nest(body_idx, by) => {
            store.add(Document::Nest(resolve_try_catch(store, body_idx), by))
        }
        Document::Flatten(body_idx) => {
            store.add(Document::Flatten(resolve_try_catch(store, body_idx)))
        }
        Document::List(vec) => todo!(),
        Document::TryCatch(try_body_idx, catch_body_idx) => {}
    }
}
