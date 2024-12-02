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

use std::env;

use argh::FromArgs;
use camino::Utf8PathBuf;
use codespan_reporting::term::{termcolor::ColorChoice, ColorArg};

/// Format Spade code
#[derive(FromArgs)]
pub struct CliOpts {
    /// coloring: auto, always, never
    #[argh(option, default = "ColorArg(ColorChoice::Auto)")]
    pub color: ColorArg,

    /// show version information
    #[argh(switch, short = 'v')]
    pub version: bool,

    #[argh(positional)]
    pub file: Utf8PathBuf,
}

impl CliOpts {
    pub fn from_env() -> Self {
        if env::args().len() == 2
            && matches!(
                env::args().nth(1).as_deref(),
                Some("-v") | Some("--version")
            )
        {
            CliOpts {
                color: ColorArg(ColorChoice::Auto),
                version: true,
                file: Default::default(),
            }
        } else {
            argh::from_env()
        }
    }
}
