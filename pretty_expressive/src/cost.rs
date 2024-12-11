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

use std::{cmp, ops::Add};

/// A cost can be added and are totally ordered. More formally, a cost is a
/// "totally ordered monoid with translational invariance" (p. 261:10).
pub trait Cost: Sized + Ord + Add<Output = Self> + Clone {
    /// The cost for formatting `length` characters at the given `column`,
    /// subject to the following invariants:
    ///
    /// 1. For any length `l` and columns `c`, `c'`, if `c <= c'`,
    ///    `Self::for_text(c, l) <= Self::for_text(c', l)`.
    /// 2. For any column `c` and lengths `l`, `l'`, `Self::for_text(c, l + l')
    ///    = Self::for_text(c + l, l')`.
    /// 3. For any column `c`, `Self::for_text(c, 0) = Self::for_text(0, 0)`.
    fn for_text(column: usize, length: usize, max_width: usize) -> Self;

    const NEWLINE_COST: Self;
}

/// "The following cost factory targets an optimality objective that minimizes
/// the sum of squared overflows over the page width limit..., and then the
/// height" (p. 261:10).
#[derive(PartialEq, Eq, Clone)]
pub struct Example3_5(usize, usize);

impl PartialOrd for Example3_5 {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Example3_5 {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self(o_a, h_a) = self;
        let Self(o_b, h_b) = other;
        o_a.cmp(h_a).then(o_b.cmp(h_b))
    }
}

impl Add for Example3_5 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let Self(o_a, h_a) = self;
        let Self(o_b, h_b) = rhs;
        Self(o_a + o_b, h_a + h_b)
    }
}

impl Cost for Example3_5 {
    fn for_text(column: usize, length: usize, max_width: usize) -> Self {
        if column + length > max_width {
            // I don't know what `a` and `b` mean:
            let a = cmp::max(max_width, column) - max_width;
            let b = column + length - cmp::max(max_width, column);
            Self(b * (2 * a + b), 0)
        } else {
            Self(0, 0)
        }
    }

    const NEWLINE_COST: Self = Self(0, 1);
}
