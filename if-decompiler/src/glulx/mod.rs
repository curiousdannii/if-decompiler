/*

Glulx
=====

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use std::collections::BTreeMap;
use std::io::Cursor;

use bytes::Buf;

use super::*;

mod disassembler;
pub mod opcodes;

pub struct GlulxState {
    pub debug_function_data: Option<BTreeMap<u32, DebugFunctionData>>,
    pub functions: BTreeMap<u32, Function>,
    pub image: Box<[u8]>,
}

impl GlulxState {
    pub fn new(image: Box<[u8]>, debug_function_data: Option<BTreeMap<u32, DebugFunctionData>>) -> GlulxState {
        GlulxState {
            debug_function_data,
            functions: BTreeMap::default(),
            image,
        }
    }

    pub fn decompile_rom(&mut self) {
        let edges = self.disassemble();
        self.mark_all_unsafe_functions(edges);
    }

    pub fn read_addr(&self, addr: u32) -> u32 {
        let mut cursor = Cursor::new(&self.image);
        cursor.set_position(addr as u64);
        cursor.get_u32()
    }
}

impl VirtualMachine for GlulxState {
    fn get_functions(&self) -> FnvHashMap<u32, FunctionSafety> {
        let mut res = FnvHashMap::default();
        for (&addr, function) in &self.functions {
            res.insert(addr, function.safety);
        }
        res
    }

    fn mark_function_as_unsafe(&mut self, addr: u32) {
        let function = self.functions.get_mut(&addr).unwrap();
        if function.safety == FunctionSafety::SafetyTBD {
            function.safety = FunctionSafety::Unsafe;
        }
    }
}

pub struct Function {
    pub addr: u32,
    pub argument_mode: FunctionArgumentMode,
    pub blocks: BTreeMap<u32, BasicBlock<Instruction>>,
    pub locals: u32,
    pub safety: FunctionSafety,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FunctionArgumentMode {
    Stack,
    Locals,
}

pub struct Instruction {
    pub addr: u32,
    pub opcode: u32,
    pub operands: Vec<Operand>,
    pub branch: Option<BranchTarget>,
    // These could be inside an Option, but we can just set them to Constants if the instruction doesn't store
    pub storer: Operand,
    pub storer2: Operand,
    pub next: u32,
}

impl VMInstruction for Instruction {
    fn addr(&self) -> u32 {
        self.addr
    }

    fn does_halt(&self) -> bool {
        opcodes::instruction_halts(self.opcode)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Operand {
    Constant(u32),
    Memory(u32),
    Stack,
    Local(u32),
    RAM(u32),
}