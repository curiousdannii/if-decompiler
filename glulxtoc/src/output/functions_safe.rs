/*

Output safe functions
=====================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use std::io::prelude::*;

use if_decompiler;

use super::*;

impl GlulxOutput {
    pub fn output_safe_functions(&self) -> std::io::Result<()> {
        let mut code_file = self.make_file("functions_safe.c")?;
        let mut header_file = self.make_file("functions_safe.h")?;

        // Output the headers
        write!(code_file, "#include \"functions_safe.h\"
#include \"glk.h\"
#include \"glulxtoc.h\"

")?;
        write!(header_file, "#include \"glk.h\"

")?;

        // Output the function bodies
        let mut safe_funcs = Vec::default();
        for (addr, function) in &self.state.functions {
            if function.safety == if_decompiler::FunctionSafety::Unsafe {
                continue;
            }

            safe_funcs.push(addr);
            let args_list = function_arguments(function.locals);
            let function_spec = format!("glui32 VM_FUNC_{}({})", addr, args_list);

            write!(code_file, "{} {{
}}

", function_spec)?;
            // And the header declaration
            writeln!(header_file, "extern glui32 VM_FUNC_{}({});", addr, args_list)?;
        }

        // Output the VM_FUNC_IS_SAFE function
        writeln!(code_file, "int VM_FUNC_IS_SAFE(glui32 addr) {{
    switch (addr) {{")?;
        for row in safe_funcs[..].chunks(5) {
            write!(code_file, "        ")?;
            let mut row_str = String::new();
            for addr in row {
                row_str.push_str(&format!("case {}: ", addr));
            }
            row_str.truncate(row_str.len() - 1);
            writeln!(code_file, "{}", row_str)?;
        }
        write!(code_file, "            return 1;
        default:
            return 0;
    }}
}}")?;

        Ok(())
    }
}

fn function_arguments(count: u32) -> String {
    let mut output = String::new();
    if count == 0 {
        return String::from("void");
    }
    for arg in 0..count {
        output.push_str(&format!("glui32 l{}, ", arg));
    }
    output.truncate(output.len() - 2);

    output
}