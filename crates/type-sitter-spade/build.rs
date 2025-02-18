use snafu::{ResultExt, Whatever};
use std::{
    env, fs,
    path::{Path, PathBuf},
};
use type_sitter_gen::generate_nodes;

const TREE_SITTER_PATH: &str = "../tree-sitter-spade";

#[snafu::report]
fn main() -> Result<(), Whatever> {
    // Common setup
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    println!("cargo::rerun-if-changed=build.rs");

    // Obligatory: in this and future lines, replace
    // `../tree-sitter-spade` with the path to your
    // grammar's folder, relative to the folder containing `Cargo.toml`
    println!("cargo::rerun-if-changed={}", TREE_SITTER_PATH);

    // To generate nodes
    let path = Path::new(TREE_SITTER_PATH).join("src/node-types.json");
    fs::write(
        out_dir.join("nodes.rs"),
        generate_nodes(path.clone())
            .whatever_context(format!(
                "Failed to generate node types from {}",
                path.to_string_lossy()
            ))?
            .into_string(),
    )
    .whatever_context(format!(
        "Failed to write node types file to {}",
        out_dir.join("nodes.rs").to_string_lossy()
    ))?;

    Ok(())
}
