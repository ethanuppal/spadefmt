// Copyright (C) 2024 Ethan Uppal.
//
// This file is part of pretty-expressive.
//
// pretty-expressive is free software: you can redistribute it and/or modify it
// under the terms of the GNU Lesser General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at your
// option) any later version. pretty-expressive is distributed in the hope that
// it will be useful, but WITHOUT ANY WARRANTY; without even the implied
// warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Lesser General Public License for more details. You should have received a
// copy of the GNU Lesser General Public License along with pretty-expressive.
// If not, see <https://www.gnu.org/licenses/>.

//! Implements [A Pretty Expressive Printer](https://dl.acm.org/doi/pdf/10.1145/3622837) without
//! the align primitive. References are made to page numbers in this PDF.

pub mod cost;
pub mod document;
pub mod interned_map;
pub mod layout;
pub mod resolve;
