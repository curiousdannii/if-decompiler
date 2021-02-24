/*

Output safe functions
=====================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use std::io::prelude::*;
use std::time::Instant;
use fnv::FnvHashMap;

use if_decompiler::*;
use glulx::*;
use glulx::opcodes;

use super::*;

impl GlulxOutput {
    pub fn output_safe_functions(&self) -> std::io::Result<()> {
        let start = Instant::now();

        let mut code_file = self.make_file("functions_safe.c")?;
        let mut header_file = self.make_file("functions_safe.h")?;

        // Output the headers
        write!(code_file, "#include \"functions_safe.h\"
#include \"glk.h\"
#include \"glulxe.h\"
#include \"glulxtoc.h\"

#define CALL_FUNC(code) (oldsp = stackptr, oldvsb = valstackbase, res = code, stackptr = oldsp, valstackbase = oldvsb, res)

")?;
        write!(header_file, "#include \"glk.h\"

")?;

        // Output the function bodies
        let mut safe_funcs: FnvHashMap<u32, Vec<u32>> = FnvHashMap::default();
        let mut highest_arg_count = 0;
        for (addr, function) in &self.state.functions {
            if function.safety == FunctionSafety::Unsafe {
                continue;
            }

            // Add to the list of safe_funcs
            match safe_funcs.get_mut(&function.locals) {
                Some(vec) => {
                    vec.push(*addr);
                },
                None => {
                    if function.locals > highest_arg_count {
                        highest_arg_count = function.locals;
                    }
                    safe_funcs.insert(function.locals, vec![*addr]);
                },
            };
            let args_list = function_arguments(function.locals, true, ",");
            let function_spec = format!("glui32 VM_FUNC_{}({})", addr, args_list);

            writeln!(code_file, "{} {{
    glui32 arg, oldsp, oldvsb, res;
    valstackbase = stackptr;", function_spec)?;
            for instruction in &function.instructions {
                writeln!(code_file, "    /* {:>3X}/{} */ {};", instruction.opcode, instruction.addr, self.output_instruction_safe(instruction))?;
            }
            writeln!(code_file, "    return 0;
}}
")?;

            // And the header declaration
            writeln!(header_file, "extern glui32 VM_FUNC_{}({});", addr, args_list)?;
        }

        // Output the VM_FUNC_ARGUMENTS_COUNT function
        writeln!(code_file, "int VM_FUNC_ARGUMENTS_COUNT(glui32 addr) {{
    switch (addr) {{")?;
        for (count, funcs) in &safe_funcs {
            for row in funcs[..].chunks(5) {
                write!(code_file, "        ")?;
                let mut row_str = String::new();
                for addr in row {
                    row_str.push_str(&format!("case {}: ", addr));
                }
                row_str.truncate(row_str.len() - 1);
                writeln!(code_file, "{}", row_str)?;
            }
            writeln!(code_file, "            return {};", count)?;
        }
        writeln!(code_file, "        default:
            return -1;
    }}
}}
")?;

        // Output the VM_CALL_SAFE_FUNCTION_WITH_STACK_ARGS function
        writeln!(code_file, "glui32 VM_CALL_SAFE_FUNCTION_WITH_STACK_ARGS(glui32 addr, glui32 count) {{
    {};", function_arguments(highest_arg_count, true, ";"))?;
        for i in 0..highest_arg_count {
            writeln!(code_file, "    if (count > {}) {{ l{} = PopStack(); }}", i, i)?;
        }
        writeln!(code_file, "    switch (addr) {{")?;
        for (count, funcs) in &safe_funcs {
            for addr in funcs {
                writeln!(code_file, "        case {}: return VM_FUNC_{}({});", addr, addr, function_arguments(*count, false, ","))?;
            }
        }
        write!(code_file, "        default: fatal_error_i(\"VM_CALL_SAFE_FUNCTION_WITH_STACK_ARGS called with non-safe function address:\", addr);
    }}
}}")?;

        let duration = start.elapsed();
        println!("Time outputting safe functions: {:?}", duration);
        Ok(())
    }

    // Output an instruction
    fn output_instruction_safe(&self, instruction: &Instruction) -> String {
        let opcode = instruction.opcode;
        let operands = self.map_operands_safe(instruction);
        let null = String::from("NULL");
        let op_a = operands.get(0).unwrap_or(&null);
        let op_b = operands.get(1).unwrap_or(&null);
        use opcodes::*;
        let body = match opcode {
            OP_CALL => self.output_call_on_stack_safe(instruction, op_a, op_b),
            OP_RETURN => format!("return {}", op_a),
            OP_TAILCALL => format!("return {}", self.output_call_on_stack_safe(instruction, op_a, op_b)),
            OP_CALLF ..= OP_CALLFIII => self.output_callf_safe(instruction, operands),
            _ => self.output_common_instruction(instruction, operands),
        };
        self.output_storer_safe(opcode, instruction.storer, body)
    }

    // Map operands into strings
    fn map_operands_safe(&self, instruction: &Instruction) -> Vec<String> {
        instruction.operands.iter().map(|&operand| self.output_operand_safe(operand)).collect()
    }

    fn output_operand_safe(&self, operand: Operand) -> String {
        use Operand::*;
        match operand {
            Constant(val) => val.to_string(),
            Memory(addr) => format!("Mem4({})", addr),
            Stack => String::from("PopStack()"),
            Local(val) => format!("l{}", val / 4),
            RAM(addr) => format!("Mem4({})", addr + self.ramstart),
        }
    }

    fn output_storer_safe(&self, opcode: u32, storer: Operand, inner: String) -> String {
        use Operand::*;
        let func = match opcode {
            opcodes::OP_COPYS => "MemW2",
            opcodes::OP_COPYB => "MemW1",
            _ => "MemW4",
        };
        match storer {
            Constant(_) => inner, // Must still output the inner code in case there are side-effects
            Memory(addr) => format!("{}({}, {})", func, addr, inner),
            Stack => format!("PushStack({})", inner),
            Local(val) => format!("l{} = {}", val / 4, inner),
            RAM(addr) => format!("{}({}, {})", func, addr + self.ramstart, inner),
        }
    }

    // Construct a call
    fn output_call_safe(&self, instruction: &Instruction, mut args: Vec<String>, is_callf: bool) -> String {
        use Operand::*;
        let callee_addr = match instruction.operands[0] {
            Constant(addr) => addr,
            _ => panic!("Dynamic callf not supported"),
        };
        let callee = self.state.functions.get(&callee_addr).unwrap();
        let provided_args = args.len();
        let callee_args = callee.locals as usize;

        // Account for extra args
        if provided_args > callee_args {
            args.truncate(callee_args);
            // First check if any of the surplus args are stack pops - we don't need to account for other types
            let mut surplus_stack_pops = 0;
            if is_callf {
                for i in callee_args..provided_args {
                    // Add 1 because we removed the callee address
                    match instruction.operands[i + 1] {
                        Stack => {
                            surplus_stack_pops += 1;
                        },
                        _ => {},
                    };
                }
            } else {
                surplus_stack_pops = provided_args - callee_args;
            }
            if surplus_stack_pops > 0 {
                let last_arg = &args[callee_args - 1];
                args[callee_args - 1] = format!("(arg = {}, stackptr -= {}, arg)", last_arg, surplus_stack_pops * 4);
            }
        }

        // Account for not enough args
        while args.len() < callee_args {
            args.push(String::from("0"));
        }

        format!("CALL_FUNC(VM_FUNC_{}({}))", callee_addr, args.join(", "))
    }

    fn output_callf_safe(&self, instruction: &Instruction, mut operands: Vec<String>) -> String {
        // Remove the address
        operands.remove(0);
        self.output_call_safe(instruction, operands, true)
    }

    fn output_call_on_stack_safe(&self, instruction: &Instruction, addr: &String, count: &String) -> String {
        use Operand::*;
        match instruction.operands[1] {
            Constant(count) => {
                let mut args = Vec::new();
                for _ in 0..count {
                    args.push(String::from("PopStack()"));
                }
                self.output_call_safe(instruction, args, false)
            },
            _ => {
                format!("CALL_FUNC(VM_CALL_SAFE_FUNCTION_WITH_STACK_ARGS({}, {}))", addr, count)
            },
        }
    }
}

fn function_arguments(count: u32, include_types: bool, separator: &str) -> String {
    let mut output = String::new();
    if count == 0 {
        return String::from(if include_types {"void"} else {""});
    }
    for arg in 0..count {
        output.push_str(&format!("{}l{}{} ", if include_types {"glui32 "} else {""}, arg, separator));
    }
    output.truncate(output.len() - 2);

    output
}