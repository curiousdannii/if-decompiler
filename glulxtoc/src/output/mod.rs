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

use if_decompiler::*;
use glulx::GlulxState;

mod functions_common;
mod functions_safe;
mod functions_unsafe;
mod image;
mod templates;

pub struct GlulxOutput {
    pub disassemble_mode: bool,
    pub have_warned_about_dynamic_branches: bool,
    pub name: String,
    pub out_dir: PathBuf,
    pub ramstart: u32,
    pub safe_functions: Vec<u32>,
    pub state: GlulxState,
    pub unsafe_functions: Vec<u32>,
    pub workspace_dir: PathBuf,
}

impl GlulxOutput {
    pub fn new(disassemble_mode: bool, name: String, out_dir: PathBuf, state: GlulxState, workspace_dir: PathBuf) -> GlulxOutput {
        let ramstart = state.read_addr(8);
        let mut safe_functions = Vec::new();
        let mut unsafe_functions = Vec::new();
        for (&addr, function) in &state.functions {
            if !disassemble_mode && function.safety == FunctionSafety::SafetyTBD {
                safe_functions.push(addr);
            }
            else {
                unsafe_functions.push(addr);
            }
        }
        GlulxOutput {
            disassemble_mode,
            have_warned_about_dynamic_branches: false,
            name,
            out_dir,
            ramstart,
            safe_functions,
            state,
            unsafe_functions,
            workspace_dir,
        }
    }

    pub fn output(&mut self) -> io::Result<()> {
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
fn safe_stack_pops(operands: &Vec<String>, in_macro: bool) -> (String, Vec<String>) {
    let safe_pops = if in_macro { 0 } else { 1 };
    let mut stack_operands = 0;
    for operand in operands {
        if operand == "PopStack()" {
            stack_operands += 1;
        }
    }
    if stack_operands <= safe_pops {
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
        if operand == "PopStack()" && op + safe_pops < stack_operands {
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
    let (prelude, new_operands) = safe_stack_pops(operands, false);
    if prelude == "" {
        return format.format(operands);
    }
    format!("({}, {})", prelude, format.format(&new_operands))
}

// Now an expression that uses a macro (such as Mem4)
fn format_safe_stack_pops_macro(format: &str, operands: &Vec<String>) -> String {
    let (prelude, new_operands) = safe_stack_pops(operands, true);
    if prelude == "" {
        return format.format(operands);
    }
    format!("({}, {})", prelude, format.format(&new_operands))
}

// And the same but for a statement
fn format_safe_stack_pops_statement(format: &str, operands: &Vec<String>) -> String {
    let (prelude, new_operands) = safe_stack_pops(operands, false);
    if prelude == "" {
        return format.format(operands);
    }
    format!("{}; {}", prelude, format.format(&new_operands))
}