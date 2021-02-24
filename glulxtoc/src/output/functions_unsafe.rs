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
use glulx::*;
use glulx::opcodes;

use super::*;

impl GlulxOutput {
    pub fn output_unsafe_functions(&self) -> std::io::Result<()> {
        let start = Instant::now();

        let mut code_file = self.make_file("functions_unsafe.c")?;

        // Output the header
        writeln!(code_file, "#include \"glk.h\"
#include \"glulxe.h\"
#include \"glulxtoc.h\"

void execute_loop(void) {{
    glui32 *arglist;
    while (1) {{
        switch (pc) {{")?;

        // Output the function bodies
        for (addr, function) in &self.state.functions {
            if function.safety == FunctionSafety::SafetyTBD {
                continue;
            }

            writeln!(code_file, "            // VM_FUNC_{}", addr)?;

            let blocks = function.basic_blocks();
            for block in blocks {
                writeln!(code_file, "            case {}:", block[0].addr)?;
                for instruction in block {
                    writeln!(code_file, "                /* {:>3X}/{} */ {};", instruction.opcode, instruction.addr, self.output_instruction_unsafe(instruction))?;
                }
            }
        }

        write!(code_file, "            default: fatal_error_i(\"Branched to invalid address:\", pc);
        }}
    }}
}}")?;

        let duration = start.elapsed();
        println!("Time outputting unsafe functions: {:?}", duration);
        Ok(())
    }

    // Output an instruction
    fn output_instruction_unsafe(&self, instruction: &Instruction) -> String {
        let opcode = instruction.opcode;
        let operands = self.map_operands_unsafe(instruction);
        let null = String::from("NULL");
        let op_a = operands.get(0).unwrap_or(&null);
        let op_b = operands.get(1).unwrap_or(&null);
        use opcodes::*;
        let body = match opcode {
            OP_CALL => self.output_call_on_stack_unsafe(instruction, op_a, op_b),
            OP_RETURN => format!("leave_function(); if (stackptr == 0) {{return;}} pop_callstub({}); break", op_a),
            OP_JUMPABS => format!("pc = {}; break", op_a),
            OP_QUIT => String::from("return"),
            _ => self.output_storer_unsafe(opcode, instruction.storer, self.output_common_instruction(instruction, operands)),
        };
        body
    }

    // Map operands into strings
    fn map_operands_unsafe(&self, instruction: &Instruction) -> Vec<String> {
        instruction.operands.iter().map(|&operand| self.output_operand_unsafe(operand)).collect()
    }

    fn output_operand_unsafe(&self, operand: Operand) -> String {
        use Operand::*;
        match operand {
            Constant(val) => val.to_string(),
            Memory(addr) => format!("Mem4({})", addr),
            Stack => String::from("PopStack()"),
            Local(addr) => format!("ReadLocal({})", addr),
            RAM(addr) => format!("Mem4({})", addr + self.ramstart),
        }
    }

    fn output_storer_unsafe(&self, opcode: u32, storer: Operand, inner: String) -> String {
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
            Local(addr) => format!("StoreLocal({}, {})", addr, inner),
            RAM(addr) => format!("{}({}, {})", func, addr + self.ramstart, inner),
        }
    }

    fn output_call_on_stack_unsafe(&self, instruction: &Instruction, addr: &String, count: &String) -> String {
        use Operand::*;
        match instruction.operands[1] {
            _ => {
                format!("arglist = pop_arguments({count}, 0); push_callstub(inst[2].desttype, inst[2].value); enter_function({addr}, {count}, arglist); break", addr=addr, count=count)
            },
        }
    }
}