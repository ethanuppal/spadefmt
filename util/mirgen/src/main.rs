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

#![forbid(unsafe_code)]

use std::{
    env, fs,
    io::{self, IsTerminal, Write},
    rc::Rc,
    sync::RwLock,
};

use snafu::{whatever, ResultExt, Whatever};
pub use spade;
use spade::{Artefacts, ModuleNamespace};
use spade_codespan_reporting::{
    files::{Files, SimpleFiles},
    term::termcolor::Buffer,
};
use spade_common::name::Path;
use spade_diagnostics::{emitter::CodespanEmitter, CodeBundle, DiagHandler};
use spade_parser::logos::Logos;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let file = env::args().nth(1).expect("missing filename input");

    const FILE_ID: usize = 0;

    let code = fs::read_to_string(&file)
        .whatever_context(format!("Failed to read file at {}", file))?;

    let mut files = SimpleFiles::new();
    let file_id = files.add(file.to_string(), code.clone());

    let diagnostic_handler = DiagHandler::new(Box::new(CodespanEmitter));

    let code_bundle = Rc::new(RwLock::new(CodeBundle { files }));

    let mut buffer = if !io::stderr().is_terminal() {
        Buffer::no_color()
    } else {
        Buffer::ansi()
    };

    let source = (
        ModuleNamespace {
            namespace: Path::from_strs(&["mirgen"]),
            base_namespace: Path::from_strs(&["mirgen"]),
            file: file.clone(),
        },
        file,
        code,
    );

    let opts = spade::Opt {
        error_buffer: &mut buffer,
        outfile: None,
        mir_output: None,
        verilator_wrapper_output: None,
        state_dump_file: None,
        item_list_file: None,
        print_type_traceback: false,
        print_parse_traceback: false,
        opt_passes: vec![],
    };

    let Ok(Artefacts {
        flat_mir_entities, ..
    }) = spade::compile(vec![source], true, opts, diagnostic_handler)
    else {
        io::stderr()
            .write_all(buffer.as_slice())
            .whatever_context("Failed to write to buffer")?;
        whatever!("Failed to compile Spade code");
    };

    for flat_mir_entity in flat_mir_entities {
        println!("{}", flat_mir_entity.0);
    }

    Ok(())
}
