/*

glulxtoc - Decompile a Glulx file into C code
=============================================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use std::path::PathBuf;
use structopt::StructOpt;

use if_decompiler;

mod output;

#[derive(StructOpt)]
#[structopt(name = "glulxtoc", about = "Decompile a Glulx file into C code")]
struct Cli {
    /// The path of the Glulxe storyfile
    #[structopt(parse(from_os_str))]
    path: PathBuf,

    /// Output folder
    #[structopt(parse(from_os_str))]
    out_dir: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();

    // Process arguments
    let storyfile = &args.path;
    let storyfile_name = storyfile.file_name().expect("storyfile should not be relative");

    let out_dir = match &args.out_dir {
        Some(path) => path.as_path(),
        None => storyfile.parent().unwrap_or(&storyfile),
    };

    // Read the storyfile
    let data = std::fs::read(storyfile)?;

    // Decompile the storyfile
    let mut decompiler = if_decompiler::glulx::GlulxState::new(data.into_boxed_slice());
    decompiler.decompile_rom();

    // Output the C files
    output::image(&decompiler.image, out_dir, storyfile_name)?;

    Ok(())
}