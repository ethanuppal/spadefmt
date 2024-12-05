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

use std::fmt::{self, Debug};

use derivative::Derivative;
use serde::Deserialize;
use string16::{string16, String16};

mod string16 {
    pub type String16 = u128;

    pub const fn string16(value: &'static str) -> String16 {
        assert!(
            value.len() <= 16,
            "string16 does not support strings with more than 8 bytes"
        );

        let mut result = 0u128;

        macro_rules! pack_bytes {
            (&mut $result:expr, $str:expr, $($idx:expr),*) => {
                $(
                    if $str.len() > $idx {
                        $result |= ($str.as_bytes()[$idx] as u128) << ($idx * 8);
                    }
                )*
            }
        }

        pack_bytes!(
            &mut result,
            value,
            0,
            1,
            2,
            3,
            4,
            5,
            6,
            7,
            8,
            9,
            10,
            11,
            12,
            13,
            14,
            15
        );

        result
    }

    pub fn to_string(str16: String16) -> String {
        let mut bytes = [0u8; 16];
        (0..16).for_each(|i| {
            bytes[i] = ((str16 >> (i * 8)) & 0xFF) as u8;
        });
        let length = bytes.iter().position(|&b| b == 0).unwrap_or(16);
        String::from_utf8_lossy(&bytes[..length]).into_owned()
    }
}

#[derive(Debug)]
pub enum BoundedIntegerParseError {
    TooLow(String16, usize, usize),
    TooHigh(String16, usize, usize),
}

impl fmt::Display for BoundedIntegerParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLow(name, value, lower_bound) => {
                write!(
                    f,
                    "cannot have {} of {}: {} is the minimum allowed",
                    string16::to_string(*name),
                    value,
                    lower_bound
                )
            }
            Self::TooHigh(name, value, upper_bound) => {
                write!(
                    f,
                    "cannot have {} of {}: {} is the maximum allowed",
                    string16::to_string(*name),
                    value,
                    upper_bound
                )
            }
        }
    }
}

/// A bounded `usize` for the `spadefmt` [`Config`]. For instance, we can have
/// `BoundedConfigUsize<1, 5, 1, { string16("error count") }>`, which is a
/// `usize` bounded between `1` and `5`, with [`Default`] value `1`, and in
/// units of "error count".
#[derive(Derivative, Deserialize)]
#[derivative(Default)]
#[derivative(Clone)]
#[derivative(Copy)]
#[serde(try_from = "usize")]
pub struct BoundedConfigUsize<
    const LOWER_BOUND: usize,
    const UPPER_BOUND: usize,
    const DEFAULT: usize,
    const UNITS: String16,
> {
    #[derivative(Default(value = "DEFAULT"))]
    inner: usize,
}

impl<
        const LOWER_BOUND: usize,
        const UPPER_BOUND: usize,
        const DEFAULT: usize,
        const UNITS: String16,
    > TryFrom<usize>
    for BoundedConfigUsize<LOWER_BOUND, UPPER_BOUND, DEFAULT, UNITS>
{
    type Error = BoundedIntegerParseError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value < LOWER_BOUND {
            Err(BoundedIntegerParseError::TooLow(UNITS, value, LOWER_BOUND))
        } else if value > UPPER_BOUND {
            Err(BoundedIntegerParseError::TooHigh(UNITS, value, UPPER_BOUND))
        } else {
            Ok(Self { inner: value })
        }
    }
}

impl<
        const LOWER_BOUND: usize,
        const UPPER_BOUND: usize,
        const DEFAULT: usize,
        const UNITS: String16,
    > Debug for BoundedConfigUsize<LOWER_BOUND, UPPER_BOUND, DEFAULT, UNITS>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<
        const LOWER_BOUND: usize,
        const UPPER_BOUND: usize,
        const DEFAULT: usize,
        const UNITS: String16,
    > From<BoundedConfigUsize<LOWER_BOUND, UPPER_BOUND, DEFAULT, UNITS>>
    for usize
{
    fn from(
        val: BoundedConfigUsize<LOWER_BOUND, UPPER_BOUND, DEFAULT, UNITS>,
    ) -> Self {
        val.inner
    }
}

#[derive(Default, Deserialize, Debug)]
pub enum FunctionSignatureStyle {
    Wide,
    Block,
    #[default]
    Tall,
}

/// Configures the behavior of `spadefmt`.
#[derive(Derivative, Deserialize, Debug)]
#[derivative(Default)]
pub struct Config {
    /// The maximum line length `spadefmt` should aim for.
    #[serde(default)]
    pub max_width: BoundedConfigUsize<
        1,
        { usize::MAX },
        100,
        { string16("character count") },
    >,

    /// The amount of spaces to indent a line.
    #[serde(default)]
    pub indent: BoundedConfigUsize<
        1,
        { usize::MAX },
        4,
        { string16("character count") },
    >,
}
