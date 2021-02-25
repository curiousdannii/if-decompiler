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
#include <math.h>

void execute_loop(void) {{
    glui32 temp0, temp1;
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

        write!(code_file, "            default:
                // Try to recover - if we are jumping into the first address of a safe function we can tailcall it
                if (VM_JUMP_CALL(pc)) {{
                    break;
                }}
                fatal_error_i(\"Branched to invalid address:\", pc);
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
            OP_JUMP => String::new(),
            OP_CALL => output_call_unsafe(op_a, op_b, instruction.storer),
            OP_RETURN => format!("leave_function(); if (stackptr == 0) {{return;}} pop_callstub({}); break", op_a),
            OP_TAILCALL => format!("VM_TAILCALL_FUNCTION({}, {}); if (stackptr == 0) {{return;}} break", op_a, op_b),
            OP_JUMPABS => String::new(),
            OP_CALLF ..= OP_CALLFIII => output_callf_unsafe(instruction, operands),
            OP_GETIOSYS => self.output_double_storer_unsafe(instruction, String::from("stream_get_iosys(&temp0, &temp1)")),
            OP_QUIT => String::from("return"),
            OP_FMOD => self.output_double_storer_unsafe(instruction, format!("OP_FMOD({}, {}, &temp0, &temp1)", op_a, op_b)),
            _ => self.output_storer_unsafe(opcode, instruction.storer, self.output_common_instruction(instruction, operands)),
        };
        self.output_branch(instruction, body)
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

    fn output_double_storer_unsafe(&self, instruction: &Instruction, inner: String) -> String {
        use Operand::*;
        let store = |storer: Operand, i: u32| {
            match storer {
                Constant(_) => String::from("NULL"),
                Memory(addr) => format!("MemW4({}, temp{})", addr, i),
                Stack => format!("PushStack(temp{})", i),
                Local(addr) => format!("StoreLocal({}, temp{})", addr, i),
                RAM(addr) => format!("MemW4({}, temp{})", addr + self.ramstart, i),
            }
        };
        format!("{}; {}; {}", inner, store(instruction.storer, 0), store(instruction.storer2, 1))
    }

    fn output_branch(&self, instruction: &Instruction, condition: String) -> String {
        use Branch::*;
        match instruction.branch {
            DoesNotBranch => condition,
            Branches(branch) => {
                let action = self.output_branch_action(instruction, branch);
                format!("if ({}) {{{}; break;}}", condition, action)
            },
            Jumps(branch) => {
                if instruction.opcode == opcodes::OP_JUMP {
                    format!("{}; break", self.output_branch_action(instruction, branch))
                } else {
                    format!("pc = {}; break", self.output_operand_unsafe(*instruction.operands.last().unwrap()))
                }
            },
        }
    }

    fn output_branch_action(&self, instruction: &Instruction, branch: BranchTarget) -> String {
        use BranchTarget::*;
        match branch {
            Dynamic => format!("if (VM_BRANCH({}, {})) {{return;}}", self.output_operand_unsafe(*instruction.operands.last().unwrap()), instruction.next),
            Absolute(addr) => format!("pc = {}", addr),
            Return(val) => format!("leave_function(); if (stackptr == 0) {{return;}} pop_callstub({})", val),
        }
    }
}

fn output_call_unsafe(addr: &String, count: &String, storer: Operand) -> String {
    format!("if (VM_CALL_FUNCTION({}, {}, {}, {})) {{break;}}", addr, count, storer_type(storer), storer_value(storer))
}

fn output_callf_unsafe(instruction: &Instruction, mut operands: Vec<String>) -> String {
    let addr = operands.remove(0);
    let count = operands.len();
    let mut inner = Vec::new();
    if count > 0 {
        // Push the arguments in reverse order
        for (i, operand) in operands.iter().enumerate() {
            inner.push(format!("StkW4(stackptr + {}, {})", (count - i - 1) * 4, operand.clone()));
        }
        inner.push(format!("stackptr += {}", count * 4));
    }
    inner.push(format!("VM_CALL_FUNCTION({}, {}, {}, {})", addr, count, storer_type(instruction.storer), storer_value(instruction.storer)));
    format!("if ({}) {{break;}}", inner.join(", "))
}

fn storer_type(storer: Operand) -> u32 {
    use Operand::*;
    match storer {
        Constant(_) => 0,
        Memory(_) | RAM(_) => 1,
        Local(_) => 2,
        Stack => 3,
    }
}

fn storer_value(storer: Operand) -> u32 {
    use Operand::*;
    match storer {
        Constant(val) | Memory(val) | Local(val) | RAM(val) => val,
        Stack => 0,
    }
}