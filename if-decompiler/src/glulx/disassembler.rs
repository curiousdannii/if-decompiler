/*

Glulx Disassembler
==================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

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

                // Strings
                0xE0 => {},
                0xE1 => {},
                0xE2 => {},

                // Unknown
                _ => {},
            }
        }
    }

    fn disassemble_function(&self, cursor: &mut Cursor<&Box<[u8]>>, addr: u32, function_mode: u8) -> Function {
        let mut safety = FunctionSafety::Safe;

        let argument_mode = match function_mode {
            0xC0 => FunctionArgumentMode::Stack,
            0xC1 => FunctionArgumentMode::Locals,
            _ => unreachable!(),
        };

        // Parse the locals formats
        let mut locals = Vec::new();
        loop {
            let local_type = cursor.get_u8();
            let count = cursor.get_u8();
            if local_type == 0 {
                break
            }
            for _ in 0..count {
                locals.push(local_type);
            }
        }

        // Parse the instructions
        let mut instructions = Vec::new();
        loop {
            match self.disassemble_instruction(cursor, addr) {
                Some(instruction) => {
                    // Update function safety from instruction safety - can only become less safe
                    if safety == FunctionSafety::Safe {
                        safety = instruction.safety;
                    }
                    else if safety == FunctionSafety::SafetyTBD && instruction.safety == FunctionSafety::Unsafe {
                        safety = FunctionSafety::Unsafe;
                    }
                    instructions.push(instruction);
                },
                None => break,
            }
        }

        Function {
            addr,
            safety,
            argument_mode,
            locals,
            instructions,
        }
    }

    fn disassemble_instruction(&self, cursor: &mut Cursor<&Box<[u8]>>, addr: u32) -> Option<Instruction> {
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

        let safety = opcodes::opcode_safety(opcode, &operands);
        Some(Instruction {
            addr,
            opcode,
            operands,
            safety,
        })
    }
}