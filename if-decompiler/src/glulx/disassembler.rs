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
    pub fn disassemble(&mut self) {
        let decoding_table = self.parse_string_decoding_table();

        let mut cursor = Cursor::new(&self.image);
        cursor.set_position(28);
        let decoding_table_addr = cursor.get_u32() as u64;
        cursor.set_position(decoding_table_addr + 8);
        let root_node_addr = cursor.get_u32();

        cursor.set_position(8);
        let ram_start = cursor.get_u32() as u64;
        // Start from the same place as glulxdump
        cursor.set_position(56);

        // Loop through the ROM
        while cursor.position() < ram_start {
            let addr = cursor.position() as u32;
            let object_type = cursor.get_u8();

            match object_type {
                // Padding
                0 => {},

                // Functions
                0xC0 | 0xC1 => {
                    self.functions.insert(addr, self.disassemble_function(&mut cursor, addr, object_type));
                },

                // Strings - just skip past them for now!
                0xE0 => {
                    while cursor.get_u8() != 0 {}
                },
                0xE2 => {
                    cursor.get_u8();
                    cursor.get_u8();
                    cursor.get_u8();
                    while cursor.get_u32() != 0 {}
                },
                // Compressed strings will take a bit more work...
                0xE1 => {
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
                _ => {},
            }
        }
    }

    // Parse the string decoding table, but only so that we can ignore compressed strings
    pub fn parse_string_decoding_table(&self) -> FnvHashMap<u32, DecodingNode> {
        let mut table = FnvHashMap::default();
        let mut cursor = Cursor::new(&self.image);
        cursor.set_position(28);
        let decoding_table_addr = cursor.get_u32() as u64;
        cursor.set_position(decoding_table_addr + 8);
        let root_node = cursor.get_u32();

        // Keep a list of nodes to process and loop through
        // I tried doing this recursively but couldn't make it work with the borrow checker
        let mut nodes_to_process = vec![root_node];
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

    fn disassemble_function(&self, cursor: &mut Cursor<&Box<[u8]>>, addr: u32, function_mode: u8) -> Function {
        println!("function addr {}", addr);
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
        entry_points.insert(cursor.position() as u32);
        let mut exit_points = FnvHashSet::default();

        // Parse the instructions
        let mut instructions = Vec::new();
        loop {
            match self.disassemble_instruction(cursor) {
                Some(instruction) => {
                    // If this instruction branches, then update the entry and exit points
                    match instruction.branch {
                        None => {},
                        Some(target) => {
                            exit_points.insert(instruction.addr);
                            match target {
                                BranchTarget::Absolute(addr) => {
                                    entry_points.insert(addr);
                                },
                                _ => {},
                            }
                        },
                    };
                    instructions.push(instruction);
                },
                None => break,
            }
        }

        Function {
            addr,
            safety: opcodes::function_safety(&instructions),
            argument_mode,
            locals,
            instructions,
            entry_points,
            exit_points,
        }
    }

    fn disassemble_instruction(&self, cursor: &mut Cursor<&Box<[u8]>>) -> Option<Instruction> {
        let addr = cursor.position() as u32;
        let opcode_byte = cursor.get_u8();

        // There's no explicit end to a function, so bail if we find a byte that looks like the beginning of a new object
        if let 0xC0 | 0xC1 | 0xE0 | 0xE1 | 0xE2 = opcode_byte {
            return None
        }

        // Unpack the variable length opcode
        let opcode = match opcode_byte {
            0 ..= 0x7F => opcode_byte as u32,
            0x80 ..= 0xBF => ((opcode_byte as u32 & 0x3F) << 8) | cursor.get_u8() as u32,
            0xC0 ..= 0xFF => ((opcode_byte as u32 & 0x3F) << 24) | ((cursor.get_u8() as u32) << 16) | cursor.get_u16() as u32,
        };

        // Extract the operands
        let mut operands = Vec::default();
        let operands_count = opcodes::operands_count(opcode, addr);
        let mut operand_types = Vec::default();
        while operand_types.len() < operands_count {
            let types = cursor.get_u8();
            operand_types.push(types & 0x0F);
            operand_types.push(types >> 4);
        }
        for i in 0..operands_count {
            let operand = match operand_types[i] {
                0 => Operand::Constant(0),
                1 => Operand::Constant(cursor.get_i8() as i32),
                2 => Operand::Constant(cursor.get_i16() as i32),
                3 => Operand::Constant(cursor.get_i32()),
                5 => Operand::Memory(cursor.get_u8() as u32),
                6 => Operand::Memory(cursor.get_u16() as u32),
                7 => Operand::Memory(cursor.get_u32()),
                8 => Operand::Stack,
                9 => Operand::Local(cursor.get_u8() as u32),
                10 => Operand::Local(cursor.get_u16() as u32),
                11 => Operand::Local(cursor.get_u32()),
                13 => Operand::RAM(cursor.get_u8() as u32),
                14 => Operand::RAM(cursor.get_u16() as u32),
                15 => Operand::RAM(cursor.get_u32()),
                x => panic!("Invalid operand mode {} in instruction {}", x, addr),
            };
            operands.push(operand);
        }

        // Calculate branch targets
        let branch = match opcodes::instruction_branches(opcode) {
            false => None,
            true => {
                Some(match *operands.last().unwrap() {
                    Operand::Constant(target) => {
                        if opcode == opcodes::OP_JUMPABS {
                            BranchTarget::Absolute(target as u32)
                        }
                        else {
                            if target == 0 || target == 1 {
                                BranchTarget::Return(target)
                            }
                            else {
                                BranchTarget::Absolute((addr as i32 + target - 2) as u32)
                            }
                        }
                    },
                    _ => BranchTarget::Dynamic,
                })
            },
        };

        Some(Instruction {
            addr,
            opcode,
            operands,
            branch,
        })
    }
}