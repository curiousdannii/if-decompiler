/*

Output unsafe functions
=======================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use std::io::prelude::*;
use std::time::Instant;

use if_decompiler::*;

use super::*;

impl GlulxOutput {
    pub fn output_unsafe_functions(&self) -> std::io::Result<()> {
        let start = Instant::now();

        let mut code_file = self.make_file("functions_unsafe.c")?;

        // Output the header
        writeln!(code_file, "#include \"glk.h\"
#include \"glulxtoc.h\"

void VM_UNSAFE_FUNCS(void) {{")?;

        // Output the function bodies
        for (addr, function) in &self.state.functions {
            if function.safety == FunctionSafety::SafetyTBD {
                continue;
            }

            writeln!(code_file, "    // VM_FUNC_{}", addr)?;
        }

        write!(code_file, "}}")?;

        let duration = start.elapsed();
        println!("Time outputting unsafe functions: {:?}", duration);
        Ok(())
    }
}