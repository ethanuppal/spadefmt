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

use std::marker::PhantomData;

use crate::interned_map::{InternedKey, InternedMap};

/// So that I can refer to `InternedMap` with `rustdoc` links.
const IMPORT_INTERNED_MAP: PhantomData<InternedMap<Document>> = PhantomData;

/// Pointer to a [`Document`] in an [`InternedMap<Document>`].
pub type DocumentRef = InternedKey<Document>;

/// Defined on pages 261:8 and 261:14.
#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Document {
    Text(String),
    Newline,
    Concat(Vec<DocumentRef>),
    Nest {
        add_indent_levels: usize,
        child: DocumentRef,
    },
    Flatten(DocumentRef),
    /// Denoted `<|>` in the paper.
    Or(DocumentRef, DocumentRef),
    // /// Denoted `<+>` in the paper.
    // AlignedConcat(DocumentRef, DocumentRef)
}
