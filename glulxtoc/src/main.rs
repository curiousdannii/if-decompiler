/*

glulxtoc - Decompile a Glulx file into C code
=============================================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use std::env;
use std::path::PathBuf;
use std::time::Instant;
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
    // Find where we are running from
    let mut workspace_dir = env::current_exe()?;
    workspace_dir.pop();
    workspace_dir.pop();
    workspace_dir.pop();

    // Process arguments
    let args = Cli::from_args();
    let mut storyfile_path = env::current_dir()?;
    storyfile_path.push(args.path);
    let name = storyfile_path.file_stem().expect("storyfile should not be relative").to_str().unwrap().to_string();

    let out_dir = match args.out_dir {
        Some(path) => path,
        None => {
            let mut path = storyfile_path.clone();
            let mut name = path.file_name().unwrap().to_os_string();
            name.push(".decompiled");
            path.pop();
            path.push(name);
            path
        }
    };

    // Read the storyfile
    let data = std::fs::read(storyfile_path)?;

    // Decompile the storyfile
    let start = Instant::now();
    let mut decompiler = if_decompiler::glulx::GlulxState::new(data.into_boxed_slice());
    decompiler.decompile_rom();
    let duration = start.elapsed();
    println!("Time disassembling the storyfile: {:?}", duration);

    // Output the C files
    let output = output::GlulxOutput {
        name,
        state: decompiler,
        out_dir,
        workspace_dir,
    };
    output.output()?;

    Ok(())
}