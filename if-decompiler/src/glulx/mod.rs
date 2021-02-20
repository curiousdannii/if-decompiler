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
use fnv::FnvHashSet;
use petgraph::graph;

use super::*;

mod disassembler;
pub mod opcodes;

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
        let mut graph = self.disassemble();

        // Add the graph edges
        graph.graph.extend_with_edges(graph.edges.iter().map(|(caller_addr, callee_addr)| {
            let caller_node = self.functions.get(caller_addr).unwrap().graph_node;
            let callee_node = self.functions.get(callee_addr).unwrap().graph_node;
            // The direction must be callee->caller, as we'll change the caller's safety if the callee is unsafe
            (callee_node, caller_node)
        }));
    }
}

// A struct for passing the graph around the functions of the disassembler
pub struct DisassemblyGraph {
    pub edges: FnvHashSet<(u32, u32)>,
    pub graph: graph::Graph<u32, ()>,
    pub unsafe_functions: Vec<u32>,
}

pub struct Function {
    pub addr: u32,
    pub argument_mode: FunctionArgumentMode,
    pub entry_points: FnvHashSet<u32>,
    pub exit_points: FnvHashSet<u32>,
    pub graph_node: graph::NodeIndex,
    pub instructions: Vec<Instruction>,
    pub locals: u32,
    pub safety: FunctionSafety,
}

// Calculate basic blocks
impl<'a> Function {
    pub fn basic_blocks(&'a self) -> Vec<&'a[Instruction]> {
        let mut basic_blocks = Vec::default();
        let mut start_index = 0;
        let instructions_count = self.instructions.len();

        for index in 0..instructions_count {
            // Finish a previous block because this one starts a new one
            if self.entry_points.contains(&self.instructions[index].addr) && index != start_index {
                basic_blocks.push(&self.instructions[start_index..(index - 1)]);
                start_index = index;
            }
            // Make a basic block because this instruction exits
            if self.exit_points.contains(&self.instructions[index].addr) {
                basic_blocks.push(&self.instructions[start_index..index]);
                start_index = index + 1;
            }
        }
        // Add a final block if needed
        if start_index < instructions_count {
            basic_blocks.push(&self.instructions[start_index..instructions_count]);
        }
        basic_blocks
    }
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

#[derive(Copy, Clone)]
pub enum Operand {
    Constant(u32),
    Memory(u32),
    Stack,
    Local(u32),
    RAM(u32),
}

#[derive(Copy, Clone, PartialEq)]
pub enum Storer {
    DoesNotStore,
    LastOperand,
    FirstOperand,
    LastTwoOperands,
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