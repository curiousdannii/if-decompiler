/*

Output the storyfile as a C array
=================================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub fn output_image(image: &[u8], out_dir: &Path, storyfile_name: &OsStr) -> std::io::Result<()> {
    // Construct the path
    let mut newname = storyfile_name.to_os_string();
    newname.push(".image.c");
    let path = out_dir.join(Path::new(&newname));

    let mut file = File::create(path)?;

    writeln!(file, "#define GLULX_IMAGE_LENGTH {}
char GLULX_IMAGE[] = {{", image.len())?;

    let image_iter = image.chunks(16);
    for row in image_iter {
        let row_text = format!("{:?}", row);
        writeln!(file, "    {},", &row_text[1..(row_text.len()-1)])?;
    }

    write!(file, "}};")?;

    Ok(())
}