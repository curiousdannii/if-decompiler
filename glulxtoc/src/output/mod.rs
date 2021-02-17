/*

Output C files
==============

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use std::fs;
use std::io;
use std::path::PathBuf;

use if_decompiler::glulx::GlulxState;

mod image;
mod templates;

pub struct GlulxOutput {
    pub name: String,
    pub state: GlulxState,
    pub out_dir: PathBuf,
    pub workspace_dir: PathBuf,
}

impl GlulxOutput {
    pub fn output(&self) -> std::io::Result<()> {
        // Make the output directory if necessary
        fs::create_dir_all(&self.out_dir)?;

        self.output_from_templates()?;
        self.output_image()?;
        Ok(())
    }

    // A little helper function for making files in the output dir
    fn make_file(&self, name: &str) -> io::Result<fs::File> {
        let mut path = self.out_dir.clone();
        path.push(name);
        fs::File::create(path)
    }
}