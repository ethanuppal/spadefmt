// Copyright (C) 2024 Ethan Uppal.
//
// This file is part of spadefmt.
//
// spadefmt is free software: you can redistribute it and/or modify it under the
// terms of the GNU General Public License as published by the Free Software
// Foundation, version 3 of the License only. spadefmt is distributed in the
// hope that it will be useful, but WITHOUT ANY WARRANTY; without even the
// implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See
// the GNU General Public License for more details. You should have received a
// copy of the GNU General Public License along with spadefmt. If not, see
// <https://www.gnu.org/licenses/>.

use std::{
    collections::HashMap,
    fmt::{self, Write},
};

use inform::common::IndentWriterCommon;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct DocumentIdx(usize);

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Document {
    Newline,
    Text(String),
    Nest(DocumentIdx, isize),
    Flatten(DocumentIdx),
    List(Vec<DocumentIdx>),
    TryCatch(DocumentIdx, DocumentIdx),
}

#[derive(Default)]
pub struct InternedDocumentStore {
    documents: Vec<Document>,
    inverse: HashMap<Document, DocumentIdx>,
}

impl InternedDocumentStore {
    pub fn add(&mut self, document: Document) -> DocumentIdx {
        if let Some(existing_idx) = self.inverse.get(&document) {
            *existing_idx
        } else {
            self.documents.push(document.clone());
            let new_idx = DocumentIdx(self.documents.len() - 1);
            self.inverse.insert(document, new_idx);
            new_idx
        }
    }

    pub fn get(&self, idx: DocumentIdx) -> &Document {
        &self.documents[idx.0]
    }

    pub fn get_mut(&mut self, idx: DocumentIdx) -> &mut Document {
        &mut self.documents[idx.0]
    }
}

pub struct DocumentDebugPrinter<'a> {
    store: &'a InternedDocumentStore,
}

impl<'a> DocumentDebugPrinter<'a> {
    pub fn new(store: &'a InternedDocumentStore) -> Self {
        Self { store }
    }

    pub fn print<W: fmt::Write>(
        &self, f: &mut inform::fmt::IndentWriter<W>, idx: DocumentIdx,
    ) -> fmt::Result {
        match self.store.get(idx) {
            Document::Newline => write!(f, "Newline"),
            Document::Text(text) => write!(f, "Text(\"{}\")", text),
            Document::Nest(body_idx, by) => {
                writeln!(f, "Nest(")?;
                f.increase_indent();
                self.print(f, *body_idx)?;
                writeln!(f, ",\n{}", by)?;
                f.decrease_indent();
                write!(f, ")")
            }
            Document::Flatten(body_idx) => {
                writeln!(f, "Flatten(")?;
                f.increase_indent();
                self.print(f, *body_idx)?;
                writeln!(f)?;
                f.decrease_indent();
                write!(f, ")")
            }
            Document::List(children) => {
                if children.is_empty() {
                    return Ok(());
                }
                writeln!(f, "List(")?;
                f.increase_indent();
                for child in children {
                    self.print(f, *child)?;
                    writeln!(f, ",")?;
                }
                f.decrease_indent();
                write!(f, ")")
            }
            Document::TryCatch(try_body, catch_body) => {
                writeln!(f, "TryCatch(")?;
                f.increase_indent();
                self.print(f, *try_body)?;
                writeln!(f, ",")?;
                self.print(f, *catch_body)?;
                writeln!(f, ",")?;
                f.decrease_indent();
                write!(f, ")")
            }
        }
    }
}
