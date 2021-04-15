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

use dyn_fmt::AsStrFormatExt;

use if_decompiler::glulx::GlulxState;

mod functions_common;
mod functions_safe;
mod functions_unsafe;
mod image;
mod templates;

pub struct GlulxOutput {
    pub disassemble_mode: bool,
    pub name: String,
    pub out_dir: PathBuf,
    pub ramstart: u32,
    pub state: GlulxState,
    pub workspace_dir: PathBuf,
}

impl GlulxOutput {
    pub fn output(&self) -> io::Result<()> {
        // Make the output directory if necessary
        fs::create_dir_all(&self.out_dir)?;

        self.output_from_templates()?;
        self.output_image()?;
        self.output_safe_functions()?;
        self.output_unsafe_functions()?;
        Ok(())
    }

    // A little helper function for making files in the output dir
    fn make_file(&self, name: &str) -> io::Result<io::BufWriter<fs::File>> {
        let mut path = self.out_dir.clone();
        path.push(name);
        let file = fs::File::create(path)?;
        Ok(io::BufWriter::new(file))
    }
}

// C says that the order function arguments are evaluated is undefined, which breaks stack pops
// This function takes a Vec of operand strings, and fixes them to ensure the order is right
fn safe_stack_pops(operands: &Vec<String>) -> (String, Vec<String>) {
    let mut stack_operands = 0;
    for operand in operands {
        if operand == "PopStack()" {
            stack_operands += 1;
        }
    }
    if stack_operands <= 1 {
        let mut new_operands = Vec::default();
        for operand in operands {
            new_operands.push(operand.clone());
        }
        return (String::new(), new_operands);
    }

    // Build the new operands
    let mut prelude = Vec::default();
    let mut new_operands = Vec::default();
    let mut op = 0;
    for operand in operands {
        if operand == "PopStack()" && op + 1 < stack_operands {
            prelude.push(format!("temp{} = PopStack()", op));
            new_operands.push(format!("temp{}", op));
            op += 1;
        }
        else {
            new_operands.push(operand.clone());
        }
    }
    (prelude.join(", "), new_operands)
}

// And then a function to use the above with a format string for an expression
fn format_safe_stack_pops_expression(format: &str, operands: &Vec<String>) -> String {
    let (prelude, new_operands) = safe_stack_pops(operands);
    if prelude == "" {
        return format.format(operands);
    }
    format!("({}, {})", prelude, format.format(&new_operands))
}

// And the same but for a statement
fn format_safe_stack_pops_statement(format: &str, operands: &Vec<String>) -> String {
    let (prelude, new_operands) = safe_stack_pops(operands);
    if prelude == "" {
        return format.format(operands);
    }
    format!("{}; {}", prelude, format.format(&new_operands))
}