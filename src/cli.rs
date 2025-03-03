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

use std::env;

use argh::FromArgs;
use camino::Utf8PathBuf;

/// Format Spade code
#[derive(Default, FromArgs)]
pub struct Opts {
    /// config file
    #[argh(option, default = "Utf8PathBuf::from(\"spadefmt.toml\")")]
    pub config: Utf8PathBuf,

    /// disable colored output
    #[argh(switch)]
    pub no_color: bool,

    /// print debug representation
    #[argh(switch)]
    pub debug: bool,

    /// show version information
    #[argh(switch, short = 'v')]
    pub version: bool,

    // file to format
    #[argh(positional)]
    pub file: Utf8PathBuf,
}

impl Opts {
    pub fn from_env() -> Self {
        if env::args().len() == 2
            && matches!(
                env::args().nth(1).as_deref(),
                Some("-v") | Some("--version")
            )
        {
            Opts {
                version: true,
                ..Default::default()
            }
        } else {
            argh::from_env()
        }
    }
}
