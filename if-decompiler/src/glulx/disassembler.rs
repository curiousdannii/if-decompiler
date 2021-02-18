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
        let mut cursor = Cursor::new(&self.image);

        cursor.set_position(8);
        let ram_start = 1000; //cursor.get_u32() as u64;
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
                    while cursor.get_u8() != 0 {};
                },
                0xE2 => {
                    cursor.get_u8();
                    cursor.get_u8();
                    cursor.get_u8();
                    while cursor.get_u32() != 0 {};
                },
                // Compressed strings will take a bit more work...
                0xE1 => {},

                // Unknown
                _ => {},
            }
        }
    }

    fn disassemble_function(&self, cursor: &mut Cursor<&Box<[u8]>>, addr: u32, function_mode: u8) -> Function {
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
                panic!("1 and 2 byte locals are not supported");
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

        // Calculate basic blocks

        Function {
            addr,
            safety: opcodes::function_safety(&instructions),
            argument_mode,
            locals,
            instructions,
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