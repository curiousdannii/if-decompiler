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
    glui32 arg, label, oldsp, oldvsb, res, temp0, temp1;
    valstackbase = stackptr;", function_spec)?;
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
        let mut input_blocks = FnvHashMap::default();
        for (&label, block) in &function.blocks {
            let mut branches = Vec::new();
            for branch in &block.branches {
                branches.push(*branch);
            }
            input_blocks.insert(label, branches);
        }

        // Run the relooper
        let block = reloop(input_blocks, *function.blocks.iter().next().unwrap().0);
        self.output_shaped_block(function, &*block, 1)
    }

    // Output a shaped block
    fn output_shaped_block(&self, function: &Function, shaped_block: &ShapedBlock<u32>, indents: usize) -> String {
        let indent = "    ".repeat(indents);
        let mut output = String::new();
        match shaped_block {
            Simple(block) => {
                let basicblock = function.blocks.get(&block.label).unwrap();
                for instruction in &basicblock.code {
                    output.push_str(&format!("{}/* {:>3X}/{} */ {}\n", indent, instruction.opcode, instruction.addr, self.output_instruction_safe(&function, &block, &instruction, indents)));
                }
                if let Some(next) = block.next.as_deref() {
                    output.push_str(&self.output_shaped_block(function, next, indents));
                }
            },
            Loop(block) => {
                output.push_str(&format!("{}while (1) {{\n{}    loop_{}_continue:\n", indent, indent, block.loop_id));
                output.push_str(&self.output_shaped_block(function, &block.inner, indents + 1));
                output.push_str(&format!("{}}}\n{}loop_{}_break:;\n", indent, indent, block.loop_id));
            },
            LoopMulti(block) => unimplemented!(),
            Multiple(block) => {
                output.push_str(&format!("{}switch (label) {{\n", indent));
                for handled in &block.handled {
                    for label in &handled.labels {
                        output.push_str(&format!("{}    case {}:\n", indent, label));
                    }
                    output.push_str(&self.output_shaped_block(function, &handled.inner, indents + 2));
                    output.push_str(&format!("{}        break;\n", indent));
                }
                output.push_str(&format!("{}}}\n", indent));
            },
        };
        output
    }

    // Output an instruction
    fn output_instruction_safe(&self, function: &Function, block: &GlulxSimpleBlock, instruction: &Instruction, indents: usize) -> String {
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
            OP_GETIOSYS => self.output_double_storer_safe(instruction, String::from("stream_get_iosys(&temp0, &temp1)")),
            OP_FMOD => self.output_double_storer_safe(instruction, format!("OP_FMOD({}, {}, &temp0, &temp1)", op_a, op_b)),
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
        use opcodes::*;
        // The double store opcodes are handled separately
        if opcode == OP_GETIOSYS || opcode == OP_FMOD {
            return inner;
        }
        use Operand::*;
        let func = match opcode {
            OP_COPYS => "MemW2",
            OP_COPYB => "MemW1",
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

    fn output_double_storer_safe(&self, instruction: &Instruction, inner: String) -> String {
        use Operand::*;
        let store = |storer: Operand, i: u32| {
            match storer {
                Constant(_) => String::from("NULL"),
                Memory(addr) => format!("MemW4({}, temp{})", addr, i),
                Stack => format!("PushStack(temp{})", i),
                Local(val) => format!("l{} = temp{}", val / 4, i),
                RAM(addr) => format!("MemW4({}, temp{})", addr + self.ramstart, i),
            }
        };
        format!("{}; {}; {}", inner, store(instruction.storer, 0), store(instruction.storer2, 1))
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

    fn output_branch_safe(&self, function: &Function, simple_block: &GlulxSimpleBlock, instruction: &Instruction, condition: String, indents: usize) -> String {
        use Branch::*;
        use BranchTarget::*;
        use opcodes::*;
        let indent = "    ".repeat(indents);
        match instruction.branch {
            DoesNotBranch => format!("{};", condition),
            Branches(branch) => {
                match branch {
                    Dynamic => panic!("Dynamic branch in safe function"),
                    Absolute(addr) => {
                        // Look in the block's branches to see if we break, continue, etc
                        if let Some(branch_mode) = simple_block.branches.get(&addr) {
                            return format!("if ({}) {{{};}}", condition, output_branchmode(branch_mode, addr))
                        }
                        // Inspect the next block
                        match simple_block.immediate.as_deref() {
                            Some(block) => {
                                match block {
                                    Simple(_) => panic!("Should not branch directly into a SimpleBlock"),
                                    Loop(_) => panic!("Should not branch directly into a LoopBlock"),
                                    LoopMulti(block) => {
                                        unimplemented!()
                                    },
                                    Multiple(block) => {
                                        let if_block = find_multiple(&block.handled, addr).unwrap();
                                        let else_block = find_multiple(&block.handled, instruction.next);
                                        let mut output = format!("if ({}) {{\n{}{}}}", condition, self.output_shaped_block(function, if_block, indents + 1), indent);
                                        if let Some(else_block) = else_block {
                                            output.push_str(&format!("\n{}else {{\n{}{}}}", indent, self.output_shaped_block(function, else_block, indents + 1), indent));
                                        }
                                        return output
                                    }
                                }
                            },
                            None => panic!("Branch with neither BranchMode nor immediate"),
                        }
                    },
                    Return(val) => return format!("if ({}) {{ return {}; }}", condition, val),
                }
            },
            Jumps(branch) => {
                if instruction.opcode == OP_JUMP {
                    match branch {
                        Dynamic => panic!("Dynamic branch in safe function"),
                        Absolute(addr) => {
                            // Look in the block's branches to see if we break, continue, etc
                            if let Some(branch_mode) = simple_block.branches.get(&addr) {
                                return format!("{};", output_branchmode(branch_mode, addr))
                            }
                            // Inspect the next block
                            match simple_block.immediate.as_deref() {
                                Some(block) => {
                                    match block {
                                        Simple(_) => return format!("/* Jumping into immediate */\n{}", self.output_shaped_block(function, block, indents)),
                                        Loop(_) => panic!("Should not branch directly into a LoopBlock"),
                                        LoopMulti(block) => {
                                            unimplemented!()
                                        },
                                        Multiple(block) => {
                                            let if_block = find_multiple(&block.handled, addr).unwrap();
                                            let else_block = find_multiple(&block.handled, instruction.next);
                                            let mut output = format!("if ({}) {{\n{}{}}}", condition, self.output_shaped_block(function, if_block, indents + 1), indent);
                                            if let Some(else_block) = else_block {
                                                output.push_str(&format!("\n{}else {{\n{}{}}}", indent, self.output_shaped_block(function, else_block, indents + 1), indent));
                                            }
                                            return output
                                        }
                                    }
                                },
                                None => panic!("Branch with neither BranchMode nor immediate"),
                            }
                        },
                        Return(val) => format!("return {};", val),
                    }
                }
                else {
                    unimplemented!();
                }
            }
        }
    }
}

fn find_multiple(handled: &Vec<HandledBlock<u32>>, label: u32) -> Option<&ShapedBlock<u32>> {
    for block in handled {
        if block.labels.contains(&label) {
            return Some(&*block.inner)
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
        LoopBreakIntoMultiple(loop_id) => format!("label = {}; goto loop_{}_break", addr, loop_id),
        LoopContinue(loop_id) => format!("goto loop_{}_continue", loop_id),
        LoopContinueMulti(loop_id) => format!("label = {}; goto loop_{}_continue", addr, loop_id),
        MergedBranch => String::from("/* Branch continues below */"),
        MergedBranchIntoMulti => format!("label = {}", addr),
    }
}