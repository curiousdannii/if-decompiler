/*

Glulx Disassembler
==================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use fnv::FnvHashSet;

use super::*;

impl GlulxState {
    pub fn disassemble(&mut self, image: &[u8]) -> FnvHashSet<(u32, u32)> {
        let decoding_table = self.parse_string_decoding_table(image);

        let mut edges = FnvHashSet::default();

        let ram_start = self.read_addr(image, 8) as u64;
        self.ramstart = ram_start as u32;
        let decoding_table_addr = self.read_addr(image, 28);
        let root_node_addr = self.read_addr(image, decoding_table_addr + 8);

        let mut cursor = Cursor::new(image);

        // If we have debug file data, use it to disassemble all the functions
        if let Some(functions) = &self.debug_function_data {
            for (&addr, func) in functions {
                cursor.set_position(addr as u64);
                let function_type = cursor.get_u8();
                self.functions.insert(func.addr, self.disassemble_function(&mut cursor, &mut edges, addr, Some(func.len), function_type));
            }
            return edges;
        }

        // Otherwise parse the file manually
        // Skip past the header
        cursor.set_position(60);

        // Loop through the ROM until the end of RAM or we find a
        while cursor.position() < ram_start {
            let addr = cursor.position() as u32;
            let object_type = cursor.get_u8();

            match object_type {
                // Padding
                0 => {},

                // Functions
                0xC0 | 0xC1 => {
                    self.functions.insert(addr, self.disassemble_function(&mut cursor, &mut edges, addr, None, object_type));
                },

                // Strings - just skip past them for now!
                0xE0 => {
                    if self.stop_on_string {
                        break;
                    }
                    while cursor.get_u8() != 0 {}
                },
                0xE2 => {
                    if self.stop_on_string {
                        break;
                    }
                    cursor.get_u8();
                    cursor.get_u8();
                    cursor.get_u8();
                    while cursor.get_u32() != 0 {}
                },
                // Compressed strings will take a bit more work...
                0xE1 => {
                    if self.stop_on_string {
                        break;
                    }

                    fn get_node<'a>(table: &'a FnvHashMap<u32, DecodingNode>, addr: u32) -> &'a DecodingNode {
                        table.get(&addr).unwrap()
                    }
                    fn get_node_branch_addresses(node: &DecodingNode) -> [u32; 2] {
                        match node {
                            DecodingNode::Branch(branch) => {
                                [branch.left, branch.right]
                            },
                            _ => panic!("Decoding node is not a branch"),
                        }
                    }

                    let root_node = get_node(&decoding_table, root_node_addr);
                    let root_branches = get_node_branch_addresses(root_node);
                    let mut left_node = root_branches[0];
                    let mut right_node = root_branches[1];

                    let mut byte = cursor.get_u8();
                    let mut bits = 8;
                    loop {
                        let bit = byte & 0x01;
                        bits -= 1;
                        byte >>= 1;
                        let node = get_node(&decoding_table, if bit == 0 {left_node} else {right_node});
                        match node {
                            DecodingNode::Terminator => {
                                break;
                            },
                            DecodingNode::Leaf => {
                                left_node = root_branches[0];
                                right_node = root_branches[1];
                            },
                            DecodingNode::Branch(branch) => {
                                left_node = branch.left;
                                right_node = branch.right;
                            },
                        }
                        if bits == 0 {
                            bits = 8;
                            byte = cursor.get_u8();
                        }
                    }
                },

                // Unknown
                _ => {
                    println!("Stopping on unknown object type {:?} at {:?}", object_type, addr);
                    break;
                },
            }
        };

        // Return the list of edges
        edges
    }

    // Parse the string decoding table, but only so that we can ignore compressed strings
    pub fn parse_string_decoding_table(&self, image: &[u8]) -> FnvHashMap<u32, DecodingNode> {
        let mut table = FnvHashMap::default();
        let mut cursor = Cursor::new(image);

        let decoding_table_addr = self.read_addr(image, 28);
        let root_node_addr = self.read_addr(image, decoding_table_addr + 8);

        // Keep a list of nodes to process and loop through
        // I tried doing this recursively but couldn't make it work with the borrow checker
        let mut nodes_to_process = vec![root_node_addr];
        loop {
            let addr = nodes_to_process.pop().unwrap();
            cursor.set_position(addr as u64);
            let node_type = cursor.get_u8();
            let node = match node_type {
                0x00 => {
                    let left = cursor.get_u32();
                    let right = cursor.get_u32();
                    nodes_to_process.push(left);
                    nodes_to_process.push(right);
                    DecodingNode::Branch(DecodingNodeBranch {
                        left,
                        right,
                    })
                },
                0x01 => DecodingNode::Terminator,
                0x02 => {
                    cursor.get_u8();
                    DecodingNode::Leaf
                },
                0x03 => {
                    while cursor.get_u8() != 0 {}
                    DecodingNode::Leaf
                },
                0x04 | 0x08 | 0x09 => {
                    cursor.get_u32();
                    DecodingNode::Leaf
                },
                0x05 => {
                    while cursor.get_u32() != 0 {}
                    DecodingNode::Leaf
                },
                0x0A | 0x0B => {
                    let _addr = cursor.get_u32();
                    let count = cursor.get_u32();
                    for _ in 0..count {
                        cursor.get_u32();
                    }
                    DecodingNode::Leaf
                }
                _ => panic!("Invalid string decoding node at {}", addr),
            };
            table.insert(addr, node);
            if nodes_to_process.len() == 0 {
                break;
            }
        }

        table
    }

    fn disassemble_function(&self, cursor: &mut Cursor<&[u8]>, edges: &mut FnvHashSet<(u32, u32)>, addr: u32, len: Option<u32>, function_mode: u8) -> Function {
        let argument_mode = match function_mode {
            0xC0 => FunctionArgumentMode::Stack,
            0xC1 => FunctionArgumentMode::Locals,
            _ => unreachable!(),
        };

        // Parse the locals formats
        let mut locals = 0;
        loop {
            let local_type = cursor.get_u8();
            let count = cursor.get_u8() as u32;
            if local_type == 0 {
                break
            }
            if local_type != 4 {
                panic!("1 and 2 byte locals are not supported in function {}", addr);
            }
            locals += count;
        }

        // Basic blocks
        let mut entry_points = FnvHashSet::default();
        let mut exit_branches = FnvHashMap::default();

        // Parse the instructions
        let end_addr = len.map(|l| addr + l);
        let mut instructions = Vec::new();
        let mut instruction_addresses = FnvHashSet::default();
        'parse_loop: loop {
            let instruction = self.disassemble_instruction(cursor);
            instruction_addresses.insert(instruction.addr);

            // If this instruction branches, then update the entry and exit points
            if let Some(target) = instruction.branch {
                match instruction.opcode {
                    opcodes::OP_JUMP | opcodes::OP_JUMPABS => {
                        let mut branch_targets = Vec::new();
                        if let BranchTarget::Absolute(addr) = target {
                            entry_points.insert(addr);
                            branch_targets.push(addr);
                        }
                        exit_branches.insert(instruction.addr, branch_targets);
                    },
                    _ => {
                        // If the branch returns then don't end a basic block here
                        // Except for @catch!
                        let returns = match target {
                            BranchTarget::Return(_) => true,
                            _ => false,
                        };
                        if !returns || instruction.opcode == opcodes::OP_CATCH {
                            entry_points.insert(instruction.next);
                            let mut branch_targets = vec![instruction.next];
                            if let BranchTarget::Absolute(addr) = target {
                                entry_points.insert(addr);
                                branch_targets.push(addr);
                            }
                            exit_branches.insert(instruction.addr, branch_targets);
                        }
                    },
                };
            }
            let opcode = instruction.opcode;

            // If this instruction calls, then add it to the edges list
            if opcodes::instruction_calls(opcode) {
                if let Operand::Constant(callee_addr) = instruction.operands[0] {
                    edges.insert((addr, callee_addr));
                }
            }

            // Add an entry point for instructions which may resume later
            if opcodes::instruction_resumes(opcode) {
                entry_points.insert(instruction.next);
            }

            instructions.push(instruction);

            // If we have an end_addr (from a debug file) then use it to determine when to stop decoding
            if let Some(end_addr) = end_addr {
                if cursor.position() as u32 == end_addr {
                    break;
                }
                continue;
            }

            if opcodes::instruction_halts(opcode) {
                // Stop parsing instructions if we don't have any pending entry_points
                // Short cut - check if the next address is an entry point
                if !entry_points.contains(&(cursor.position() as u32)) {
                    // Otherwise check if any entry points haven't already been parsed
                    for _ in entry_points.difference(&instruction_addresses) {
                        continue 'parse_loop;
                    }

                    // And check for an unreachable instruction
                    let final_addr = cursor.position();
                    let potential_opcode = decode_opcode(cursor);
                    cursor.set_position(final_addr);
                    // Check for 0 first, as it shouldn't be interpreted as a NOP
                    if potential_opcode == 0 {
                        break;
                    }
                    match opcodes::operands_count(potential_opcode) {
                        Some(_) => {
                            entry_points.insert(final_addr as u32);
                            continue 'parse_loop;
                        },
                        None => break,
                    };
                }
            }
        }

        let safety = self.function_safety(addr, &instructions);
        let blocks = calculate_basic_blocks(instructions, entry_points, exit_branches);

        Function {
            addr,
            argument_mode,
            blocks,
            locals,
            safety,
        }
    }

    fn disassemble_instruction(&self, cursor: &mut Cursor<&[u8]>) -> Instruction {
        use Operand::*;

        let addr = cursor.position() as u32;
        let opcode = decode_opcode(cursor);

        // Extract the operands
        let mut operands = Vec::default();
        let operands_count = opcodes::operands_count(opcode).expect(&format!("Unknown opcode {} at address {}", opcode, addr)) as usize;
        let mut operand_types = Vec::default();
        while operand_types.len() < operands_count {
            let types = cursor.get_u8();
            operand_types.push(types & 0x0F);
            operand_types.push(types >> 4);
        }
        for i in 0..operands_count {
            let operand = match operand_types[i] {
                0 => Constant(0),
                1 => Constant(cursor.get_i8() as i32 as u32),
                2 => Constant(cursor.get_i16() as i32 as u32),
                3 => Constant(cursor.get_u32()),
                5 => Memory(cursor.get_u8() as u32),
                6 => Memory(cursor.get_u16() as u32),
                7 => Memory(cursor.get_u32()),
                8 => Stack,
                9 => Local(cursor.get_u8() as u32),
                10 => Local(cursor.get_u16() as u32),
                11 => Local(cursor.get_u32()),
                13 => RAM(cursor.get_u8() as u32),
                14 => RAM(cursor.get_u16() as u32),
                15 => RAM(cursor.get_u32()),
                x => panic!("Invalid operand mode {} in instruction {}", x, addr),
            };
            operands.push(operand);
        }

        // Calculate branch targets
        use BranchTarget::*;
        let calc_branch = || -> BranchTarget {
            match *operands.last().unwrap() {
                Constant(target) => {
                    if opcode == opcodes::OP_JUMPABS {
                        Absolute(target)
                    }
                    else {
                        if target == 0 || target == 1 {
                            Return(target)
                        }
                        else {
                            Absolute((cursor.position() as i32 + target as i32 - 2) as u32)
                        }
                    }
                },
                _ => Dynamic,
            }
        };
        let branch = match opcodes::instruction_branches(opcode) {
            true => Some(calc_branch()),
            false => None,
        };

        // Extract the storer(s) - in reverse order (makes it simpler for OP_FMOD)
        use opcodes::StoreMode::*;
        let (storer2, storer) = match opcodes::instruction_stores(opcode) {
            DoesNotStore => (Operand::Constant(0), Operand::Constant(0)),
            LastOperand => (Operand::Constant(0), operands.pop().unwrap()),
            LastTwoOperands => (operands.pop().unwrap(), operands.pop().unwrap()),
        };

        Instruction {
            addr,
            opcode,
            operands,
            branch,
            storer,
            storer2,
            next: cursor.position() as u32,
        }
    }

    // Check the function safety overrides
    fn function_safety(&self, addr: u32, instructions: &Vec<Instruction>) -> FunctionSafety {
        if let Some(functions) = &self.safe_function_overides {
            if functions.contains(&addr) {
                return FunctionSafety::SafetyTBD;
            }
        }
        if let Some(functions) = &self.unsafe_function_overides {
            if functions.contains(&addr) {
                return FunctionSafety::Unsafe;
            }
        }
        opcodes::function_safety(instructions)
    }
}

// Decode a variable length opcode
fn decode_opcode(cursor: &mut Cursor<&[u8]>) -> u32 {
    let opcode_byte = cursor.get_u8();
    match opcode_byte {
        0 ..= 0x7F => opcode_byte as u32,
        0x80 ..= 0xBF => ((opcode_byte as u32 & 0x3F) << 8) | cursor.get_u8() as u32,
        0xC0 ..= 0xFF => ((opcode_byte as u32 & 0x3F) << 24) | ((cursor.get_u8() as u32) << 16) | cursor.get_u16() as u32,
    }
}

pub enum DecodingNode {
    Branch(DecodingNodeBranch),
    Leaf,
    Terminator,
}

pub struct DecodingNodeBranch {
    pub left: u32,
    pub right: u32,
}