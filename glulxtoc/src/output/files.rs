/*

Create files
============

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use super::*;
use std::time::Instant;

impl GlulxOutput {
    pub fn output_from_templates(&self, file_length: usize) -> std::io::Result<()> {
        let start = Instant::now();

        let templates = [
            "CMakeLists.txt",
            "glulxtoc.h",
            "runtime.c",
            "unixstrt.c",
        ];
        let replacements = [
            ["IMAGE_LENGTH_VALUE", &file_length.to_string()],
            ["NAME", &self.name],
            ["OUTDIR", self.out_dir.to_str().unwrap()],
            ["WORKSPACE", self.workspace_dir.to_str().unwrap()],
        ];

        let mut template_dir = self.workspace_dir.clone();
        template_dir.push("glulxtoc/src/output/templates");

        for template_name in &templates {
            let mut template_path = template_dir.clone();
            template_path.push(template_name);
            let mut file = fs::read_to_string(template_path)?;

            for replacement in &replacements {
                file = file.replace(replacement[0], replacement[1]);
            }

            let mut output_path = self.out_dir.clone();
            output_path.push(template_name);
            fs::write(output_path, file)?;
        }

        let duration = start.elapsed();
        println!("Time outputting files from templates: {:?}", duration);
        Ok(())
    }
}