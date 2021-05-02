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
use FunctionSafety::*;
use glulx::*;
use Operand::*;
use glulx::opcodes;

use super::*;

impl GlulxOutput {
    pub fn output_unsafe_functions(&mut self) -> std::io::Result<()> {
        let start = Instant::now();

        let mut code_file = self.make_file("functions_unsafe.c")?;

        // Output the header
        writeln!(code_file, "#include \"glk.h\"
#include \"glulxe.h\"
#include \"glulxtoc.h\"
#include <math.h>

void execute_loop(void) {{
    glui32 temp0, temp1, temp2, temp3, temp4, temp5;
    while (1) {{
        switch (pc) {{")?;

        // Output the function bodies
        for addr in &self.unsafe_functions {
            let function = &self.state.functions[addr];

            if function.safety == UnsafeDynamicBranches && self.state.debug_function_data.is_none() && !self.have_warned_about_dynamic_branches {
                println!("Warning ❗ This Glulx file features dynamic branches or jumps; please provide an Inform debug file.");
                self.have_warned_about_dynamic_branches = true;
            }

            let name = self.state.debug_function_data.as_ref().map_or(String::new(), |functions| format!(" ({})", functions.get(addr).unwrap().name));
            writeln!(code_file, "            // VM Function {}{}", addr, name)?;

            for (label, block) in &function.blocks {
                if function.safety != UnsafeDynamicBranches {
                    writeln!(code_file, "            case {}:", label)?;
                }
                for instruction in &block.code {
                    let instruction_label = if function.safety == UnsafeDynamicBranches { format!("case {}: ", instruction.addr) } else { String::new() };
                    writeln!(code_file, "                {}/* {:>3X}/{} */ {};", instruction_label, instruction.opcode, instruction.addr, self.output_instruction_unsafe(&instruction))?;
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
        use opcodes::*;
        let body = match opcode {
            OP_CALL => self.output_call_unsafe(&operands, instruction),
            OP_RETURN => format!("temp0 = {}; leave_function(); if (stackptr == 0) {{return;}} pop_callstub(temp0); break", op_a),
            OP_TAILCALL => format_safe_stack_pops_statement("VM_TAILCALL_FUNCTION({}, {}); if (stackptr == 0) {{return;}} break", &operands),
            OP_CATCH => format!("if (OP_CATCH({}, {}, {}, {})) {{return;}} break", storer_type(instruction.operands[0]), self.storer_value(instruction.operands[0]), operands[1], instruction.next),
            OP_THROW => format!("temp0 = {}; stackptr = {}; pop_callstub(temp0); break", op_a, operands[1]),
            OP_COPYS => self.output_copys_unsafe(instruction),
            OP_COPYB => self.output_copyb_unsafe(instruction),
            OP_STREAMCHAR => format!("if (OP_STREAMX(0, {}, {})) {{break;}}", op_a, instruction.next),
            OP_STREAMNUM => format!("if (OP_STREAMX(1, {}, {})) {{break;}}", op_a, instruction.next),
            OP_STREAMSTR => format!("if (OP_STREAMX(2, {}, {})) {{break;}}", op_a, instruction.next),
            OP_STREAMUNICHAR => format!("if (OP_STREAMX(3, {}, {})) {{break;}}", op_a, instruction.next),
            OP_CALLF ..= OP_CALLFIII => self.output_callf_unsafe(instruction, operands),
            OP_GETIOSYS => self.output_double_storer_unsafe(instruction, String::from("stream_get_iosys(&temp0, &temp1)")),
            OP_RESTART => String::from("vm_restart(); break"),
            OP_SAVE => format!("OP_SAVE({}, {}, {}, {})", op_a, instruction.next, storer_type(instruction.storer), self.storer_value(instruction.storer)),
            OP_RESTORE => format!("if (OP_RESTORE({}, {}, {})) {{break;}}", op_a, storer_type(instruction.storer), self.storer_value(instruction.storer)),
            OP_SAVEUNDO => format!("OP_SAVEUNDO({}, {}, {})", instruction.next, storer_type(instruction.storer), self.storer_value(instruction.storer)),
            OP_RESTOREUNDO => format!("if (OP_RESTOREUNDO({}, {})) {{break;}}", storer_type(instruction.storer), self.storer_value(instruction.storer)),
            OP_QUIT => String::from("return"),
            OP_FMOD => self.output_double_storer_unsafe(instruction, format_safe_stack_pops_expression("OP_FMOD({}, {}, &temp0, &temp1)", &operands)),
            _ => self.output_storer_unsafe(instruction.storer, self.output_common_instruction(instruction, operands)),
        };
        self.output_branch_unsafe(instruction, body)
    }

    // Map operands into strings
    fn map_operands_unsafe(&self, instruction: &Instruction) -> Vec<String> {
        instruction.operands.iter().map(|&operand| self.output_operand_unsafe(operand)).collect()
    }

    fn output_operand_unsafe(&self, operand: Operand) -> String {
        match operand {
            Constant(val) => val.to_string(),
            Memory(addr) => format!("Mem4({})", addr),
            Stack => String::from("PopStack()"),
            Local(addr) => format!("ReadLocal({})", addr),
            RAM(addr) => format!("Mem4({})", addr + self.ramstart),
        }
    }

    fn output_storer_unsafe(&self, storer: Operand, inner: String) -> String {
        match storer {
            Constant(_) => inner, // Must still output the inner code in case there are side-effects
            Memory(addr) => format!("store_operand(1, {}, {})", addr, inner),
            Stack => format!("PushStack({})", inner),
            Local(addr) => format!("store_operand(2, {}, {})", addr, inner),
            RAM(addr) => format!("store_operand(1, {}, {})", addr + self.ramstart, inner),
        }
    }

    fn output_double_storer_unsafe(&self, instruction: &Instruction, inner: String) -> String {
        let store = |storer: Operand, i: u32| {
            match storer {
                Constant(_) => String::from("NULL"),
                Memory(addr) => format!("store_operand(1, {}, temp{})", addr, i),
                Stack => format!("PushStack(temp{})", i),
                Local(addr) => format!("store_operand(2, {}, temp{})", addr, i),
                RAM(addr) => format!("store_operand(1, {}, temp{})", addr + self.ramstart, i),
            }
        };
        format!("{}; {}; {}", inner, store(instruction.storer, 0), store(instruction.storer2, 1))
    }

    fn output_branch_unsafe(&self, instruction: &Instruction, condition: String) -> String {
        use opcodes::*;
        match instruction.branch {
            None => condition,
            Some(target) => match instruction.opcode {
                OP_CATCH => condition,
                OP_JUMP => format!("{}; break", self.output_branch_action_unsafe(instruction, target)),
                OP_JUMPABS => format!("pc = {}; break", self.output_operand_unsafe(*instruction.operands.last().unwrap())),
                _ => format!("if ({}) {{{}; break;}}", condition, self.output_branch_action_unsafe(instruction, target)),
            },
        }
    }

    fn output_branch_action_unsafe(&self, instruction: &Instruction, branch: BranchTarget) -> String {
        use BranchTarget::*;
        match branch {
            Dynamic => format!("if (VM_BRANCH({}, {})) {{return;}}", self.output_operand_unsafe(*instruction.operands.last().unwrap()), instruction.next),
            Absolute(addr) => format!("pc = {}", addr),
            Return(val) => format!("temp0 = {}; leave_function(); if (stackptr == 0) {{return;}} pop_callstub(temp0)", val),
        }
    }

    fn output_call_unsafe(&self, operands: &Vec<String>, instruction: &Instruction) -> String {
        let (prelude, new_operands) = safe_stack_pops(operands, false);
        let prelude_out = if prelude == "" { String::new() } else { format!("{}; ", prelude) };
        format!("{}if (VM_CALL_FUNCTION({}, {}, {}, {}, {})) {{break;}}", prelude_out, new_operands[0], new_operands[1], storer_type(instruction.storer), self.storer_value(instruction.storer), instruction.next)
    }

    fn output_callf_unsafe(&self, instruction: &Instruction, operands: Vec<String>) -> String {
        let (prelude, new_operands) = safe_stack_pops(&operands, false);
        let inner = match operands.len() {
            1 => format!("VM_CALL_FUNCTION({}, 0", operands[0]),
            2 => format!("OP_CALLFI({}", new_operands.join(", ")),
            3 => format!("OP_CALLFII({}", new_operands.join(", ")),
            4 => format!("OP_CALLFIII({}", new_operands.join(", ")),
            _ => unreachable!(),
        };
        let prelude_out = if prelude == "" { String::new() } else { format!("{}; ", prelude) };
        format!("{}if ({}, {}, {}, {})) {{break;}}", prelude_out, inner, storer_type(instruction.storer), self.storer_value(instruction.storer), instruction.next)
    }

    fn output_copys_unsafe(&self, instruction: &Instruction) -> String {
        let inner = match instruction.operands[0] {
            Constant(val) => format!("{} & 0xFFFF", val),
            Memory(addr) => format!("Mem2({})", addr),
            Stack => String::from("PopStack() & 0xFFFF"),
            Local(addr) => format!("Stk2({} + localsbase)", addr),
            RAM(addr) => format!("Mem2({})", addr + self.ramstart),
        };
        match instruction.operands[1] {
            Constant(_) => inner,
            Memory(addr) => format!("store_operand_s(1, {}, {})", addr, inner),
            Stack => format!("PushStack({})", inner),
            Local(addr) => format!("store_operand_s(2, {}, {})", addr, inner),
            RAM(addr) => format!("store_operand_s(1, {}, {})", addr + self.ramstart, inner),
        }
    }

    fn output_copyb_unsafe(&self, instruction: &Instruction) -> String {
        let inner = match instruction.operands[0] {
            Constant(val) => format!("{} & 0xFF", val),
            Memory(addr) => format!("Mem1({})", addr),
            Stack => String::from("PopStack() & 0xFF"),
            Local(addr) => format!("Stk1({} + localsbase)", addr),
            RAM(addr) => format!("Mem1({})", addr + self.ramstart),
        };
        match instruction.operands[1] {
            Constant(_) => inner,
            Memory(addr) => format!("store_operand_b(1, {}, {})", addr, inner),
            Stack => format!("PushStack({})", inner),
            Local(addr) => format!("store_operand_b(2, {}, {})", addr, inner),
            RAM(addr) => format!("store_operand_b(1, {}, {})", addr + self.ramstart, inner),
        }
    }

    fn storer_value(&self, storer: Operand) -> u32 {
        match storer {
            Constant(_) | Stack => 0,
            Memory(val) | Local(val) => val,
            RAM(val) => val + self.ramstart,
        }
    }
}

fn storer_type(storer: Operand) -> u32 {
    match storer {
        Constant(_) => 0,
        Memory(_) | RAM(_) => 1,
        Local(_) => 2,
        Stack => 3,
    }
}