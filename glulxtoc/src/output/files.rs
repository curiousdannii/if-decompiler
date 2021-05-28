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
    pub fn output_from_templates(&self, data: &[u8]) -> std::io::Result<()> {
        let start = Instant::now();

        // Output the image
        let mut output_path = self.out_dir.clone();
        output_path.push("image.data");
        fs::write(output_path, data)?;

        // Output the Glulx sources
        let glulx_sources = [
            ("files.c", include_str!("../../../upstream/glulxe/files.c")),
            ("float.c", include_str!("../../../upstream/glulxe/float.c")),
            ("funcs.c", include_str!("../../../upstream/glulxe/funcs.c")),
            ("gestalt.c", include_str!("../../../upstream/glulxe/gestalt.c")),
            ("gestalt.h", include_str!("../../../upstream/glulxe/gestalt.h")),
            ("glkop.c", include_str!("../../../upstream/glulxe/glkop.c")),
            ("glulxe.h", include_str!("../../../upstream/glulxe/glulxe.h")),
            ("heap.c", include_str!("../../../upstream/glulxe/heap.c")),
            ("main.c", include_str!("../../../upstream/glulxe/main.c")),
            ("opcodes.h", include_str!("../../../upstream/glulxe/opcodes.h")),
            ("operand.c", include_str!("../../../upstream/glulxe/operand.c")),
            ("osdepend.c", include_str!("../../../upstream/glulxe/osdepend.c")),
            ("profile.c", include_str!("../../../upstream/glulxe/profile.c")),
            ("search.c", include_str!("../../../upstream/glulxe/search.c")),
            ("serial.c", include_str!("../../../upstream/glulxe/serial.c")),
            ("string.c", include_str!("../../../upstream/glulxe/string.c")),
            ("unixstrt.h", include_str!("../../../upstream/glulxe/unixstrt.h")),
            ("vm.c", include_str!("../../../upstream/glulxe/vm.c")),
        ];

        // Make the output directory if necessary
        let mut glulxe_path = self.out_dir.clone();
        glulxe_path.push("glulxe");
        fs::create_dir_all(&glulxe_path)?;
        for glulxe_name in &glulx_sources {
            let mut output_path = glulxe_path.clone();
            output_path.push(glulxe_name.0);
            fs::write(output_path, glulxe_name.1)?;
        }

        // Output the template files
        let templates = [
            ("CMakeLists.txt", include_str!("templates/CMakeLists.txt")),
            ("glulxtoc.h", include_str!("templates/glulxtoc.h")),
            ("runtime.c", include_str!("templates/runtime.c")),
            ("unixstrt.c", include_str!("templates/unixstrt.c")),
        ];
        let replacements = [
            ["GLULXE_FILES", &glulx_sources.iter().map(|(file, _)| *file).collect::<Vec<&str>>().join(" ")],
            ["IMAGE_LENGTH_VALUE", &data.len().to_string()],
            ["EXENAME", &self.name],
        ];

        for template_name in &templates {
            let mut file = String::from(template_name.1);
            for replacement in &replacements {
                file = file.replace(replacement[0], replacement[1]);
            }

            let mut output_path = self.out_dir.clone();
            output_path.push(template_name.0);
            fs::write(output_path, file)?;
        }

        let duration = start.elapsed();
        println!("Time outputting files from templates: {:?}", duration);
        Ok(())
    }
}