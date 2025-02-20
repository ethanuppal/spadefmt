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
    io::{self, IsTerminal},
    rc::Rc,
    sync::RwLock,
};

use snafu::{whatever, OptionExt, ResultExt, Whatever};
pub use spade;
use spade_codespan_reporting::{files::SimpleFiles, term::termcolor::Buffer};
use spade_diagnostics::{emitter::CodespanEmitter, CodeBundle, DiagHandler};
use spade_parser::logos::Logos;
use spadefmt::{
    cli::Opts,
    config::Config,
    document,
    document_builder::DocumentBuilder,
    resolve_try_catch::{resolve_try_catch, PrintingContext},
};
use type_sitter::{Node, Parser as TSParser};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let opts = Opts::from_env();

    if opts.version {
        println!(
            "{} {}",
            env::args().next().expect("no program name"),
            env!("CARGO_PKG_VERSION")
        );
        println!();
        print!(include_str!("../resources/version.txt"));

        return Ok(());
    }

    const FILE_ID: usize = 0;

    let code = fs::read_to_string(&opts.file)
        .whatever_context(format!("Failed to read file at {}", opts.file))?;

    let mut files = SimpleFiles::new();
    files.add(opts.file.to_string(), code.clone());

    let diagnostic_handler = DiagHandler::new(Box::new(CodespanEmitter));

    let code_bundle = Rc::new(RwLock::new(CodeBundle { files }));

    let mut buffer = if opts.no_color || !io::stderr().is_terminal() {
        Buffer::no_color()
    } else {
        Buffer::ansi()
    };

    let mut error_handler = spade::ErrorHandler {
        failed: false,
        error_buffer: &mut buffer,
        diag_handler: diagnostic_handler,
        code: code_bundle,
    };

    let mut parser = spade_parser::Parser::new(
        spade_parser::lexer::TokenKind::lexer(&code),
        FILE_ID,
    );

    if let Err(error) = parser.top_level_module_body() {
        error_handler.report(&error);
        for error in &parser.errors {
            error_handler.report(error);
        }
        whatever!("Exiting due to errors");
    }

    let mut tree_sitter_parser = TSParser::new(
        &tree_sitter_spade::language().into(),
    )
    .whatever_context("Failed to set tree-sitter parser language to Spade")?;
    let root = tree_sitter_parser
        .parse(code, None)
        .expect("Internal Spade bug: tree-sitter could not parse code accepted by the Spade parser.
Please file a bug report at <https://gitlab.com/spade-lang/tree-sitter-spade>.");
    let type_sitter_tree = type_sitter_spade::SourceFile::try_from_raw(
        root.root_node(),
    )
        .expect("Internal Spade bug: tree-sitter parser parsed top level as a node that isn't SourceFile");

    let test_contents = fs::read_to_string(opts.config).whatever_context(
        format!("Failed to read config file at {}", opts.config),
    )?;
    let test_config = toml::from_str::<Config>(&test_contents)
        .whatever_context("Failed to decode config")?;

    let indent = test_config.indent.inner;
    let cursor = type_sitter_tree.walk();
    let (mut document_store, root_idx) =
        DocumentBuilder::new(test_config.indent.inner as isize)
            .build_root(&root, cursor);

    if opts.debug {
        let mut buffer = String::new();
        let mut f = inform::fmt::IndentWriter::new(&mut buffer, indent);
        document::debug_print(&document_store, &mut f, root_idx)
            .whatever_context("Failed to print document")?;
        println!("{}", buffer);
        return Ok(());
    }

    let new_root_idx = resolve_try_catch(
        &mut document_store,
        root_idx,
        &mut PrintingContext::new(test_config.max_width.inner),
    );

    let mut buffer = String::new();
    let mut f = inform::fmt::IndentWriter::new(&mut buffer, indent);
    document::print_resolved(&document_store, &mut f, new_root_idx, false)
        .whatever_context("Failed to print document")?;
    println!("{}", buffer);

    Ok(())
}
