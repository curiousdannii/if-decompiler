/*

Output safe functions
=====================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use std::io::prelude::*;
use std::time::Instant;

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

")?;
        write!(header_file, "#include \"glk.h\"

")?;

        // Output the function bodies
        let mut safe_funcs = Vec::default();
        for (addr, function) in &self.state.functions {
            if function.safety == FunctionSafety::Unsafe {
                continue;
            }

            safe_funcs.push(addr);
            let args_list = function_arguments(function.locals);
            let function_spec = format!("glui32 VM_FUNC_{}({})", addr, args_list);

            writeln!(code_file, "{} {{", function_spec)?;
            for instruction in &function.instructions {
                writeln!(code_file, "    {}", self.output_instruction(instruction))?;
            }
            writeln!(code_file, "}}
")?;

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

        let duration = start.elapsed();
        println!("Time outputting safe functions: {:?}", duration);
        Ok(())
    }

    // Output an instruction
    fn output_instruction(&self, instruction: &Instruction) -> String {
        let opcode = instruction.opcode;
        let operands = self.map_operands(instruction);
        let null = String::from("0");
        let op_a = operands.get(0).unwrap_or(&null);
        let op_b = operands.get(1).unwrap_or(&null);
        use opcodes::*;
        let body = match opcode {
            OP_NOP => String::new(),
            OP_ADD => self.args_join(operands, " + "),
            OP_SUB => self.args_join(operands, " - "),
            OP_MUL => self.args_join(operands, " * "),
            OP_DIV => self.runtime("OP_DIV", operands),
            OP_MOD => self.runtime("OP_MOD", operands),
            OP_NEG => format!("-((glsi32) {})", op_a),
            OP_BITAND => self.args_join(operands, " & "),
            OP_BITOR => self.args_join(operands, " | "),
            OP_BITXOR => self.args_join(operands, " ^ "),
            OP_BITNOT => format!("~{}", op_a),
            OP_SHIFTL => self.runtime("OP_SHIFTL", operands),
            OP_USHIFTR => self.runtime("OP_USHIFTR", operands),
            OP_SSHIFTR => self.runtime("OP_SSHIFTR", operands),
            OP_JUMP => String::new(), // Handle elsewhere!
            OP_JZ => format!("{} == 0", op_a),
            OP_JNZ => format!("{} != 0", op_a),
            OP_JEQ => format!("{} == {}", op_a, op_b),
            OP_JNE => format!("{} != {}", op_a, op_b),
            OP_JLT => format!("(glsi32) {} < (glsi32) {}", op_a, op_b),
            OP_JGT => format!("(glsi32) {} > (glsi32) {}", op_a, op_b),
            OP_JLE => format!("(glsi32) {} <= (glsi32) {}", op_a, op_b),
            OP_JGE => format!("(glsi32) {} >= (glsi32) {}", op_a, op_b),
            OP_JLTU => format!("{} < {}", op_a, op_b),
            OP_JGTU => format!("{} > {}", op_a, op_b),
            OP_JLEU => format!("{} <= {}", op_a, op_b),
            OP_JGEU => format!("{} >= {}", op_a, op_b),

            OP_RETURN => format!("return {}", op_a),
            _ => String::from("0"),
        };
        use glulx::Storer::*;
        let body_with_storer = match instruction_stores(opcode) {
            DoesNotStore => body,
            LastOperand => self.output_storer(*instruction.operands.last().unwrap(), body),
            FirstOperand => self.output_storer(instruction.operands[0], body),
            // TODO!
            LastTwoOperands => body,
        };
        format!("/* {}/{} */ {};", instruction.opcode, instruction.addr, body_with_storer)
    }

    // Map operands into strings
    fn map_operands(&self, instruction: &Instruction) -> Vec<String> {
        use Storer::*;
        match opcodes::instruction_stores(instruction.opcode)
        {
            DoesNotStore => &instruction.operands[..],
            LastOperand => &instruction.operands[..(instruction.operands.len() - 1)],
            FirstOperand => &instruction.operands[1..],
            LastTwoOperands => &instruction.operands[..(instruction.operands.len() - 2)],
        }.iter().map(|&operand| self.output_operand(operand)).collect()
    }

    fn output_operand(&self, operand: Operand) -> String {
        use Operand::*;
        match operand {
            Constant(val) => val.to_string(),
            Memory(addr) => format!("Mem4({})", addr),
            Stack => String::from("PopStack()"),
            Local(val) => format!("l{}", val / 4),
            RAM(addr) => format!("Mem4({})", addr + self.ramstart),
        }
    }

    fn output_storer(&self, storer: Operand, inner: String) -> String {
        use Operand::*;
        match storer {
            Constant(_) => String::new(),
            Memory(addr) => format!("MemW4({}, {})", addr, inner),
            Stack => format!("PushStack({})", inner),
            Local(val) => format!("l{} = {}", val / 4, inner),
            RAM(addr) => format!("MemW4({}, {})", addr + self.ramstart, inner),
        }
    }

    // And some functions which can be used by functions_unsafe.rs too
    pub fn args_join(&self, operands: Vec<String>, joiner: &str) -> String {
        match operands.len() {
            0 => String::new(),
            1 => format!("{}", operands[0]),
            2 => format!("{}{}{}", operands[0], joiner, operands[1]),
            _ => operands.join(joiner),
        }
    }

    pub fn args(&self, operands: Vec<String>) -> String {
        self.args_join(operands, ", ")
    }

    pub fn runtime(&self, name: &str, operands: Vec<String>) -> String {
        format!("{}({})", name, self.args(operands))
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