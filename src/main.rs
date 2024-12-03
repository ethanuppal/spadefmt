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
    env, fmt, fs,
    io::{self, IsTerminal, Write},
    rc::Rc,
    sync::RwLock,
};

use camino::Utf8Path;
use codespan_reporting::{
    files::SimpleFiles,
    term::termcolor::{Buffer, BufferWriter, ColorChoice},
};
use inform::io::GenericIndentFormatter;
use logos::Logos;
use spadefmt::{
    cli::CliOpts,
    config::Config,
    format_streams::{self, Theme},
    render::Context,
    with_context::WithContext,
};

fn read_file(path: &Utf8Path) -> io::Result<String> {
    let raw_contents =
        fs::read(path).with_context(format!("failed to read {}", path))?;
    String::from_utf8(raw_contents)
        .map_err(io::Error::other)
        .with_context("Failed to decode file contents as UTF-8")
}

fn new_output_buffer(color: ColorChoice) -> Buffer {
    if !env::var("NO_COLOR").unwrap_or_default().is_empty() {
        Buffer::no_color()
    } else {
        match color {
            ColorChoice::Always | ColorChoice::AlwaysAnsi => Buffer::ansi(),
            ColorChoice::Auto => {
                if io::stdout().is_terminal() {
                    Buffer::ansi()
                } else {
                    Buffer::no_color()
                }
            }
            ColorChoice::Never => Buffer::no_color(),
        }
    }
}

fn new_error_handler<'a>(
    error_buffer: &'a mut Buffer, file: &Utf8Path, contents: String,
) -> spade::ErrorHandler<'a> {
    let mut files = SimpleFiles::new();
    files.add(file.to_string(), contents);

    let diag_handler = spade_diagnostics::DiagHandler::new(Box::new(
        spade_diagnostics::emitter::CodespanEmitter,
    ));

    let code = Rc::new(RwLock::new(spade_diagnostics::CodeBundle { files }));

    spade::ErrorHandler {
        failed: false,
        error_buffer,
        diag_handler,
        code,
    }
}

fn driver(opts: CliOpts, error_buffer: &mut Buffer) -> io::Result<()> {
    const FILE_ID: usize = 0;

    let contents = read_file(&opts.file)?;

    let mut errors =
        new_error_handler(error_buffer, &opts.file, contents.clone());

    let mut parser = spade_parser::Parser::new(
        spade_parser::lexer::TokenKind::lexer(&contents),
        FILE_ID,
    );

    let top = parser.top_level_module_body().map_err(|error| {
        errors.report(&error);
        for error in &parser.errors {
            errors.report(error);
        }
        io::Error::other(error)
    })?;

    let test_contents = read_file("test.toml".into())?;
    let test_config = toml::de::from_str::<Config>(&test_contents)
        .map_err(io::Error::other)
        .with_context("failed to decode config")?;

    println!("{:?}", test_config);

    let buffer_writer = BufferWriter::stdout(ColorChoice::Always);
    let mut buffer = buffer_writer.buffer();

    let f = GenericIndentFormatter::new(&mut buffer, 4);
    let mut stream =
        format_streams::indent_formatter::IndentFormatterStream::new(
            Theme::idk(),
            f,
        );

    Context::new(&mut stream, &test_config)
        .render(&top)
        .map_err(io::Error::other)?;

    buffer_writer.print(&buffer)?;
    println!();

    Ok(())
}

fn main() -> io::Result<()> {
    let opts = CliOpts::from_env();

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

    let mut error_buffer = new_output_buffer(opts.color.0);

    driver(opts, &mut error_buffer)
        .inspect_err(|_| {
            let _ = io::stderr().write_all(error_buffer.as_slice());
        })
        .or_else(|error| {
            println!("{}", error);
            Ok(())
        })
}
