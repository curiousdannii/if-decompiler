/*

Output the storyfile as a C array
=================================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use std::io::prelude::*;

use super::*;

impl GlulxOutput {
    pub fn output_image(&self) -> std::io::Result<()> {
        let mut file = self.make_file("image.c")?;

        writeln!(file, "char GLULX_IMAGE[] = {{")?;

        let image_iter = self.state.image.chunks(16);
        for row in image_iter {
            let row_text = format!("{:?}", row);
            writeln!(file, "    {},", &row_text[1..(row_text.len()-1)])?;
        }

        write!(file, "}};")?;

        Ok(())
    }
}