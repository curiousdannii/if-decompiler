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

")?;
        write!(header_file, "#include \"glk.h\"

")?;

        // Output the function bodies
        let mut highest_arg_count = 0;
        let mut varargs_functions = Vec::new();
        for addr in &self.safe_functions {
            let function = &self.state.functions[addr];
            if function.locals > highest_arg_count {
                highest_arg_count = function.locals;
            }
            if function.argument_mode == FunctionArgumentMode::Stack {
                varargs_functions.push(*addr);
            }

            let args_list = if function.argument_mode == FunctionArgumentMode::Stack { String::from("void") } else { function_arguments(function.locals, true, ",") };
            let function_spec = format!("glui32 VM_FUNC_{}({})", addr, args_list);
            let name_comment = self.state.debug_function_data.as_ref().map_or(String::new(), |functions| format!("// VM Function {} ({})\n", addr, functions.get(addr).unwrap().name));

            writeln!(code_file, "{}{} {{
    glui32 arg, label = 0, oldsp, oldvsb, res, temp0, temp1, temp2, temp3, temp4, temp5;", name_comment, function_spec)?;
            if function.argument_mode == FunctionArgumentMode::Stack {
                writeln!(code_file, "    glui32 {};", function_arguments(function.locals, false, ","))?;
            } else {
                writeln!(code_file, "    valstackbase = stackptr;")?;
            }
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
        for row in self.safe_functions.chunks(5) {
            write!(code_file, "        ")?;
            let mut row_str = String::new();
            for addr in row {
                row_str.push_str(&format!("case {}: ", addr));
            }
            row_str.truncate(row_str.len() - 1);
            writeln!(code_file, "{}", row_str)?;
        }
        writeln!(code_file, "            return 1;
        default:
            return 0;
    }}
}}
")?;

        // Output the VM_FUNC_IS_SAFE_VARARGS function
        writeln!(code_file, "int VM_FUNC_IS_SAFE_VARARGS(glui32 addr) {{
    switch (addr) {{")?;
        for row in varargs_functions.chunks(5) {
            write!(code_file, "        ")?;
            let mut row_str = String::new();
            for addr in row {
                row_str.push_str(&format!("case {}: ", addr));
            }
            row_str.truncate(row_str.len() - 1);
            writeln!(code_file, "{}", row_str)?;
        }
        writeln!(code_file, "            return 1;
        default:
            return 0;
    }}
}}
")?;

        // Output the VM_CALL_SAFE_FUNCTION_WITH_STACK_ARGS function
        writeln!(code_file, "glui32 VM_CALL_SAFE_FUNCTION_WITH_STACK_ARGS(glui32 addr, glui32 count) {{
    {};
    if (VM_FUNC_IS_SAFE_VARARGS(addr)) {{
        PushStack(count);
    }}
    else {{", function_arguments(highest_arg_count, true, ";"))?;
        for i in 0..highest_arg_count {
            writeln!(code_file, "        if (count > {}) {{ l{} = PopStack(); }}", i, i)?;
        }
        writeln!(code_file, "    }}\n    switch (addr) {{")?;
        for addr in &self.safe_functions {
            let function = &self.state.functions[addr];
            let args_list = if function.argument_mode == FunctionArgumentMode::Stack { String::new() } else { function_arguments(function.locals, false, ",") };
            writeln!(code_file, "        case {}: return VM_FUNC_{}({});", addr, addr, args_list)?;
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

        // Vararg functions
        if callee.argument_mode == FunctionArgumentMode::Stack {
            let (processed_args, pre_pushed_args) = match instruction.opcode {
                opcodes::OP_CALLFI ..= opcodes::OP_CALLFIII => {
                    let (prelude, new_operands) = safe_stack_pops(&args, true);
                    let pushed_args: Vec<String> = new_operands.iter().rev().map(|arg| format!("PushStack({})", arg)).collect();
                    if prelude == "" {
                        (format!("{}, ", pushed_args.join(", ")), 0)
                    }
                    else {
                        (format!("{}, {}, ", prelude, pushed_args.join(", ")), 0)
                    }
                },
                _ => {
                    (String::new(), provided_args)
                },
            };
            return format!("CALL_FUNC_VARARGS(({}PushStack({}), VM_FUNC_{}()), {})", processed_args, provided_args, callee_addr, pre_pushed_args);
        }

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
                let callee_addr = match instruction.operands[0] {
                    Constant(addr) => addr,
                    _ => panic!("Dynamic callf not supported at {:?}", instruction.addr),
                };
                let callee = &self.state.functions[&callee_addr];
                if callee.argument_mode == FunctionArgumentMode::Stack {
                    format!("(arg = {}, CALL_FUNC_VARARGS((PushStack(arg), VM_CALL_SAFE_FUNCTION_WITH_STACK_ARGS({}, arg)), arg))", count, addr)
                }
                else {
                    format!("CALL_FUNC(VM_CALL_SAFE_FUNCTION_WITH_STACK_ARGS({}, {}))", addr, count)
                }
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
                        // Handle OP_JUMP: it should have its action in the branches map, or jump into a SimpleBlock immediate
                        if instruction.opcode == OP_JUMP {
                            if let Some(branch_mode) = simple_block.branches.get(&addr) {
                                return format!("{};", output_branchmode(branch_mode, addr));
                            }
                            if let Some(immediate_block) = simple_block.immediate.as_deref_mut() {
                                if let Simple(_) = immediate_block {
                                    let output = format!("/* Jumping into immediate */\n{}", self.output_shaped_block(function, immediate_block, indents));
                                    simple_block.immediate = None;
                                    return output;
                                }
                            }
                        }

                        if let Some(Multiple(ref mut multiple_block)) = simple_block.immediate.as_deref_mut() {
                            // Check if the next instruction is in the immediate block
                            if let Some(next_block_index) = find_multiple(&multiple_block.handled, instruction.next) {
                                // if-else with both blocks in handled
                                if let Some(if_block_index) = find_multiple(&multiple_block.handled, addr) {
                                    assert!(multiple_block.handled.len() == 2, "Unhandled multiple block");
                                    let output = format!("if ({}) {{\n{}{}}}\n{}else {{\n{}{}}}", condition, self.output_multiple(function, &mut multiple_block.handled, if_block_index, indents + 1), indent, indent, self.output_multiple(function, &mut multiple_block.handled, next_block_index, indents + 1), indent);
                                    simple_block.immediate = None;
                                    return output;
                                }

                                // A simple if branch, where the branch target is a MergedBranch
                                if let Some(MergedBranch) = simple_block.branches.get(&addr) {
                                    assert!(multiple_block.handled.len() == 1, "Unhandled multiple block");
                                    let output = format!("if (!({})) {{\n{}{}}}", condition, self.output_multiple(function, &mut multiple_block.handled, next_block_index, indents + 1), indent);
                                    simple_block.immediate = None;
                                    return output;
                                }

                                // Some other kind of branch action
                                if let Some(branch_mode) = simple_block.branches.get(&addr) {
                                    assert!(multiple_block.handled.len() == 1, "Unhandled multiple block");
                                    let output = format!("if ({}) {{\n{}    {};\n{}}}\n{}else {{\n{}{}}}", condition, indent, output_branchmode(branch_mode, addr), indent, indent, self.output_multiple(function, &mut multiple_block.handled, next_block_index, indents + 1), indent);
                                    simple_block.immediate = None;
                                    return output;
                                }
                            }

                            // Otherwise the branch target could be in immediate, and the next in the branches map
                            if let Some(target_block_index) = find_multiple(&multiple_block.handled, addr) {
                                if let Some(branch_mode) = simple_block.branches.get(&instruction.next) {
                                    assert!(multiple_block.handled.len() == 1, "Unhandled multiple block");
                                    let output = format!("if ({}) {{\n{}{}}}\n{}else {{\n{}    {};\n{}}}", condition, self.output_multiple(function, &mut multiple_block.handled, target_block_index, indents + 1), indent, indent, indent, output_branchmode(branch_mode, instruction.next), indent);
                                    simple_block.immediate = None;
                                    return output;
                                }
                            }
                        }

                        // Both target and next are in the branches map
                        if let Some(target_branch_mode) = simple_block.branches.get(&addr) {
                            if let Some(next_branch_mode) = simple_block.branches.get(&instruction.next) {
                                return format!("if ({}) {{\n{}    {};\n{}}}\n{}else {{\n{}    {};\n{}}}", condition, indent, output_branchmode(target_branch_mode, addr), indent, indent, indent, output_branchmode(next_branch_mode, instruction.next), indent);
                            }
                        }

                        panic!("Unsupported branch at address {}, branching to {:?}, next {}\nBlock: {:?}", instruction.addr, target, instruction.next, simple_block);
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

    fn output_multiple(&self, function: &Function, handled: &mut Vec<HandledBlock<u32>>, index: usize, indents: usize) -> String {
        let ref mut block = handled.get_mut(index).unwrap().inner;
        self.output_shaped_block(function, block, indents)
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

fn find_multiple(handled: &Vec<HandledBlock<u32>>, label: u32) -> Option<usize> {
    for (index, block) in handled.iter().enumerate() {
        if block.labels.contains(&label) {
            return Some(index)
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