/*

Output safe functions
=====================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use std::collections::BTreeMap;
use std::io::prelude::*;
use std::time::Instant;

use fnv::FnvHashMap;

use if_decompiler::*;
use glulx::*;
use Operand::*;
use glulx::opcodes;
use relooper::*;
use BranchMode::*;
use ShapedBlock::*;

use super::*;

type GlulxSimpleBlock = SimpleBlock<u32>;

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
#include <math.h>

#define CALL_FUNC(code) (oldsp = stackptr, oldvsb = valstackbase, res = code, stackptr = oldsp, valstackbase = oldvsb, res)

")?;
        write!(header_file, "#include \"glk.h\"

")?;

        // Output the function bodies
        let mut safe_funcs: FnvHashMap<u32, Vec<u32>> = FnvHashMap::default();
        let mut highest_arg_count = 0;
        for (addr, function) in &self.state.functions {
            if self.disassemble_mode {
                break;
            }

            if function.safety != FunctionSafety::SafetyTBD {
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
            let name_comment = self.state.debug_function_data.as_ref().map_or(String::new(), |functions| format!("// VM Function {} ({})\n", addr, functions.get(addr).unwrap().name));

            writeln!(code_file, "{}{} {{
    glui32 arg, label, oldsp, oldvsb, res, temp0, temp1, temp2, temp3, temp4, temp5;
    valstackbase = stackptr;", name_comment, function_spec)?;
            code_file.write(self.output_function_body(function).as_bytes())?;
            writeln!(code_file, "    return 0;
}}
")?;

            // And the header declaration
            writeln!(header_file, "extern glui32 VM_FUNC_{}({});", addr, args_list)?;
        }

        // Output the VM_FUNC_IS_SAFE function
        writeln!(code_file, "int VM_FUNC_IS_SAFE(glui32 addr) {{
    switch (addr) {{")?;
        for (_, funcs) in &safe_funcs {
            for row in funcs[..].chunks(5) {
                write!(code_file, "        ")?;
                let mut row_str = String::new();
                for addr in row {
                    row_str.push_str(&format!("case {}: ", addr));
                }
                row_str.truncate(row_str.len() - 1);
                writeln!(code_file, "{}", row_str)?;
            }
        }
        writeln!(code_file, "            return 1;
        default:
            return 0;
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

    // Output a function
    fn output_function_body(&self, function: &Function) -> String {
        // Prepare a map of block labels and branches
        let mut input_blocks = BTreeMap::default();
        for (&label, block) in &function.blocks {
            let mut branches = Vec::new();
            for branch in &block.branches {
                branches.push(*branch);
            }
            input_blocks.insert(label, branches);
        }

        // Run the relooper
        let mut block = reloop(input_blocks, *function.blocks.iter().next().unwrap().0);
        self.output_shaped_block(function, &mut *block, 1)
    }

    // Output a shaped block
    fn output_shaped_block(&self, function: &Function, shaped_block: &mut ShapedBlock<u32>, indents: usize) -> String {
        let indent = "    ".repeat(indents);
        let mut output = String::new();
        match shaped_block {
            Simple(block) => {
                let basicblock = function.blocks.get(&block.label).unwrap();
                for instruction in &basicblock.code {
                    output.push_str(&format!("{}/* {:>3X}/{} */ {}\n", indent, instruction.opcode, instruction.addr, self.output_instruction_safe(&function, block, &instruction, indents)));
                }
                if let Some(immediate) = block.immediate.as_deref_mut() {
                    output.push_str(&self.output_shaped_block(function, immediate, indents));
                }
                if let Some(next) = block.next.as_deref_mut() {
                    output.push_str(&self.output_shaped_block(function, next, indents));
                }
            },
            Loop(block) => {
                output.push_str(&format!("{}while (1) {{\n{}    loop_{}_continue:\n", indent, indent, block.loop_id));
                output.push_str(&self.output_shaped_block(function, &mut *block.inner, indents + 1));
                output.push_str(&format!("{}}}\n{}loop_{}_break:;\n", indent, indent, block.loop_id));
                if let Some(next) = block.next.as_deref_mut() {
                    output.push_str(&self.output_shaped_block(function, next, indents));
                }
            },
            Multiple(block) => {
                output.push_str(&format!("{}switch (label) {{\n", indent));
                for handled in block.handled.iter_mut() {
                    for label in &handled.labels {
                        output.push_str(&format!("{}    case {}:\n", indent, label));
                    }
                    output.push_str(&self.output_shaped_block(function, &mut handled.inner, indents + 2));
                    output.push_str(&format!("{}        break;\n", indent));
                }
                output.push_str(&format!("{}}}\n", indent));
            },
        };
        output
    }

    // Output an instruction
    fn output_instruction_safe(&self, function: &Function, block: &mut GlulxSimpleBlock, instruction: &Instruction, indents: usize) -> String {
        let opcode = instruction.opcode;
        let operands = self.map_operands_safe(instruction);
        let null = String::from("NULL");
        let op_a = operands.get(0).unwrap_or(&null);
        let op_b = operands.get(1).unwrap_or(&null);
        use opcodes::*;
        let body = match opcode {
            // TODO: Check if call funcs need better stackpop protection
            OP_CALL => self.output_call_on_stack_safe(instruction, op_a, op_b),
            OP_RETURN => format!("return {}", op_a),
            OP_TAILCALL => format!("return {}", self.output_call_on_stack_safe(instruction, op_a, op_b)),
            OP_COPYS => self.output_copys_safe(instruction),
            OP_COPYB => self.output_copyb_safe(instruction),
            OP_STREAMCHAR => format!("(*stream_char_handler)({} & 0xFF)", op_a),
            OP_STREAMNUM => format!("stream_num((glsi32) {}, FALSE, 0)", op_a),
            OP_STREAMSTR => format!("stream_string({}, 0, 0)", op_a),
            OP_STREAMUNICHAR => format!("(*stream_unichar_handler)({})", op_a),
            OP_CALLF ..= OP_CALLFIII => self.output_callf_safe(instruction, operands),
            OP_GETIOSYS => self.output_double_storer_safe(instruction, String::from("stream_get_iosys(&temp0, &temp1)")),
            OP_FMOD => self.output_double_storer_safe(instruction, format_safe_stack_pops_expression("OP_FMOD({}, {}, &temp0, &temp1)", &operands)),
            _ => self.output_common_instruction(instruction, operands),
        };
        let body_with_storer = self.output_storer_safe(opcode, instruction.storer, body);
        self.output_branch_safe(function, block, instruction, body_with_storer, indents)
    }

    // Map operands into strings
    fn map_operands_safe(&self, instruction: &Instruction) -> Vec<String> {
        instruction.operands.iter().map(|&operand| self.output_operand_safe(operand)).collect()
    }

    fn output_operand_safe(&self, operand: Operand) -> String {
        match operand {
            Constant(val) => val.to_string(),
            Memory(addr) => format!("Mem4({})", addr),
            Stack => String::from("PopStack()"),
            Local(val) => format!("l{}", val / 4),
            RAM(addr) => format!("Mem4({})", addr + self.ramstart),
        }
    }

    fn output_storer_safe(&self, opcode: u32, storer: Operand, inner: String) -> String {
        use opcodes::*;
        // The double store opcodes are handled separately
        if opcode == OP_GETIOSYS || opcode == OP_FMOD {
            return inner;
        }
        match storer {
            Constant(_) => inner, // Must still output the inner code in case there are side-effects
            Memory(addr) => format!("store_operand(1, {}, {})", addr, inner),
            Stack => format!("PushStack({})", inner),
            Local(val) => format!("l{} = {}", val / 4, inner),
            RAM(addr) => format!("store_operand(1, {}, {})", addr + self.ramstart, inner),
        }
    }

    fn output_double_storer_safe(&self, instruction: &Instruction, inner: String) -> String {
        let store = |storer: Operand, i: u32| {
            match storer {
                Constant(_) => String::from("NULL"),
                Memory(addr) => format!("store_operand(1, {}, temp{})", addr, i),
                Stack => format!("PushStack(temp{})", i),
                Local(val) => format!("l{} = temp{}", val / 4, i),
                RAM(addr) => format!("store_operand(1, {}, temp{})", addr + self.ramstart, i),
            }
        };
        format!("{}; {}; {}", inner, store(instruction.storer, 0), store(instruction.storer2, 1))
    }

    // Construct a call
    fn output_call_safe(&self, instruction: &Instruction, mut args: Vec<String>, is_callf: bool) -> String {
        let callee_addr = match instruction.operands[0] {
            Constant(addr) => addr,
            _ => panic!("Dynamic callf not supported at {:?}", instruction.addr),
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

    fn output_branch_safe(&self, function: &Function, simple_block: &mut GlulxSimpleBlock, instruction: &Instruction, condition: String, indents: usize) -> String {
        use BranchTarget::*;
        use opcodes::*;
        let indent = "    ".repeat(indents);
        match instruction.branch {
            None => format!("{};", condition),
            Some(target) => {
                match target {
                    Dynamic => panic!("Dynamic branch in safe function at {:?}", instruction.addr),
                    Absolute(addr) => {
                        // A simple if branch
                        if let Some(MergedBranch) = simple_block.branches.get(&addr) {
                            if let Some(immediate_block) = simple_block.immediate.as_deref_mut() {
                                if let Multiple(ref mut multiple_block) = immediate_block {
                                    if multiple_block.handled.len() == 1 {
                                        let branch_a_block_opt = find_multiple(&mut multiple_block.handled, instruction.next);
                                        if let Some(mut branch_a_block) = branch_a_block_opt {
                                            let output = format!("if (!({})) {{\n{}{}}}", condition, self.output_shaped_block(function, &mut branch_a_block, indents + 1), indent);
                                            simple_block.immediate = None;
                                            return output;
                                        }
                                    }
                                }
                            }
                        }

                        // Look in the block's branches to see if we break, continue, etc
                        if let Some(branch_mode) = simple_block.branches.get(&addr) {
                            return match instruction.opcode {
                                OP_JUMP => format!("{};", output_branchmode(branch_mode, addr)),
                                OP_JUMPABS => unimplemented!("OP_JUMPABS branch not yet supported"),
                                _ => {
                                    let mut output = format!("if ({}) {{{};}}", condition, output_branchmode(branch_mode, addr));
                                    // See if we can extract the next block out of an immediate Multiple
                                    if let Some(immediate_block) = simple_block.immediate.as_deref_mut() {
                                        if let Multiple(ref mut multiple_block) = immediate_block {
                                            if multiple_block.handled.len() == 1 {
                                                let branch_a_block_opt = find_multiple(&mut multiple_block.handled, instruction.next);
                                                if let Some(mut branch_a_block) = branch_a_block_opt {
                                                    output.push('\n');
                                                    output.push_str(&self.output_shaped_block(function, &mut branch_a_block, indents));
                                                    simple_block.immediate = None;
                                                }
                                            }
                                        }
                                    }
                                    output
                                },
                            };
                        }

                        // Inspect the next block
                        match simple_block.immediate.as_deref_mut() {
                            Some(mut immediate_block) => {
                                match immediate_block {
                                    Simple(_) => {
                                        match instruction.opcode {
                                            OP_JUMP => {
                                                let output = format!("/* Jumping into immediate */\n{}", self.output_shaped_block(function, &mut immediate_block, indents));
                                                simple_block.immediate = None;
                                                output
                                            },
                                            OP_JUMPABS => unimplemented!("OP_JUMPABS branch not yet supported"),
                                            _ => panic!("Should not branch directly into a SimpleBlock"),
                                        }
                                    },
                                    Loop(_) => panic!("Should not branch directly into a LoopBlock"),
                                    Multiple(block) => {
                                        let mut if_block = find_multiple(&mut block.handled, addr).unwrap();
                                        let mut output = format!("if ({}) {{\n{}{}}}", condition, self.output_shaped_block(function, &mut if_block, indents + 1), indent);
                                        let else_block_option = find_multiple(&mut block.handled, instruction.next);
                                        if let Some(mut else_block) = else_block_option {
                                            output.push_str(&format!("\n{}else {{\n{}{}}}", indent, self.output_shaped_block(function, &mut else_block, indents + 1), indent));
                                        }
                                        simple_block.immediate = None;
                                        output
                                    },
                                }
                            },
                            None => panic!("Branch with neither BranchMode nor immediate"),
                        }
                    },
                    Return(val) => match instruction.opcode {
                        OP_JUMP => format!("return {};", val),
                        OP_JUMPABS => unimplemented!("OP_JUMPABS branch not yet supported"),
                        _ => format!("if ({}) {{ return {}; }}", condition, val),
                    },
                }
            },
        }
    }

    fn output_copys_safe(&self, instruction: &Instruction) -> String {
        let inner = match instruction.operands[0] {
            Constant(val) => format!("{} & 0xFFFF", val),
            Memory(addr) => format!("Mem2({})", addr),
            Stack => String::from("PopStack() & 0xFFFF"),
            Local(val) => format!("l{} & 0xFFFF", val / 4),
            RAM(addr) => format!("Mem2({})", addr + self.ramstart),
        };
        match instruction.operands[1] {
            Constant(_) => inner,
            Memory(addr) => format!("store_operand_s(1, {}, {})", addr, inner),
            Stack => format!("PushStack({})", inner),
            Local(val) => format!("l{} = (l{} & 0xFFFF0000) | {}", val, val, inner),
            RAM(addr) => format!("store_operand_s(1, {}, {})", addr + self.ramstart, inner),
        }
    }

    fn output_copyb_safe(&self, instruction: &Instruction) -> String {
        let inner = match instruction.operands[0] {
            Constant(val) => format!("{} & 0xFF", val),
            Memory(addr) => format!("Mem1({})", addr),
            Stack => String::from("PopStack() & 0xFF"),
            Local(val) => format!("l{} & 0xFF", val / 4),
            RAM(addr) => format!("Mem1({})", addr + self.ramstart),
        };
        match instruction.operands[1] {
            Constant(_) => inner,
            Memory(addr) => format!("store_operand_b(1, {}, {})", addr, inner),
            Stack => format!("PushStack({})", inner),
            Local(val) => format!("l{} = (l{} & 0xFFFFFF00) | {}", val, val, inner),
            RAM(addr) => format!("store_operand_b(1, {}, {})", addr + self.ramstart, inner),
        }
    }
}

fn find_multiple(handled: &mut Vec<HandledBlock<u32>>, label: u32) -> Option<&mut ShapedBlock<u32>> {
    for block in handled.iter_mut() {
        if block.labels.contains(&label) {
            return Some(&mut *block.inner)
        }
    }
    None
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

fn output_branchmode(branch_mode: &BranchMode, addr: u32) -> String {
    match branch_mode {
        LoopBreak(loop_id) => format!("goto loop_{}_break", loop_id),
        LoopBreakIntoMulti(loop_id) => format!("label = {}; goto loop_{}_break", addr, loop_id),
        LoopContinue(loop_id) => format!("goto loop_{}_continue", loop_id),
        LoopContinueIntoMulti(loop_id) => format!("label = {}; goto loop_{}_continue", addr, loop_id),
        MergedBranch => format!("/* Branch to {} continues below */", addr),
        MergedBranchIntoMulti => format!("label = {} /* Branch continues below */", addr),
    }
}