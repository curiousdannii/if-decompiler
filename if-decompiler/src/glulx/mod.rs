/*

Glulx
=====

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use bytes::Buf;
use std::io::Cursor;
use fnv::FnvHashMap;

use super::*;

mod disassembler;
mod opcodes;

pub struct GlulxState {
    pub image: Box<[u8]>,
    pub functions: FnvHashMap<u32, Function>,
}

impl GlulxState {
    pub fn new(image: Box<[u8]>) -> GlulxState {
        GlulxState {
            image,
            functions: FnvHashMap::default(),
        }
    }

    pub fn decompile_rom(&mut self) {
        self.disassemble();
    }
}

pub struct Function {
    pub addr: u32,
    pub safety: FunctionSafety,
    pub argument_mode: FunctionArgumentMode,
    pub locals: u32,
    pub instructions: Vec<Instruction>,
}

pub enum FunctionArgumentMode {
    Stack,
    Locals,
}

pub struct Instruction {
    pub addr: u32,
    pub opcode: u32,
    pub operands: Vec<Operand>,
    pub branch: Option<BranchTarget>,
}

pub enum Operand {
    Constant(i32),
    Memory(u32),
    Stack,
    Local(u32),
    RAM(u32),
}