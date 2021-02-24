/*

if-decompiler - core library
===============================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use fnv::FnvHashSet;
use petgraph::{graph, visit};

pub mod glulx;

// Function safety refers to whether or not a function can be compiled and run without worrying about its locals and stack being part of the savestate
// Unsafe functions need to be compiled such that they can be serialised and restored
// SafetyTBD functions have not yet been determined. At the end of decompilation any remaining SafetyTBD functions will be assumed safe.
#[derive(Copy, Clone, PartialEq)]
pub enum FunctionSafety {
    Unsafe,
    SafetyTBD,
}

// A struct for passing around a graph of function dependencies
pub struct DisassemblyGraph {
    pub edges: FnvHashSet<(u32, u32)>,
    pub graph: graph::Graph<u32, ()>,
    pub unsafe_functions: Vec<u32>,
}

// Now a trait for generalising over our VMs
pub trait VirtualMachine {
    fn get_function_graph_node(&self, addr: u32) -> graph::NodeIndex;
    fn mark_function_as_unsafe(&mut self, addr: u32);

    fn mark_all_unsafe_functions(&mut self, mut graph: DisassemblyGraph) {
        // First add the graph edges
        graph.graph.extend_with_edges(graph.edges.iter().map(|(caller_addr, callee_addr)| {
            let caller_node = self.get_function_graph_node(*caller_addr);
            let callee_node = self.get_function_graph_node(*callee_addr);
            // The direction must be callee->caller, as we'll change the caller's safety if the callee is unsafe
            (callee_node, caller_node)
        }));

        // Now walk the function graph, marking each caller as Unsafe
        let mut dfs = visit::Dfs::empty(&graph.graph);
        dfs.stack = graph.unsafe_functions.iter().map(|addr| self.get_function_graph_node(*addr)).collect();
        while let Some(node_index) = dfs.next(&graph.graph) {
            let addr = graph.graph[node_index];
            self.mark_function_as_unsafe(addr);
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Branch {
    DoesNotBranch,
    Branches(BranchTarget),
    Jumps(BranchTarget),
}

#[derive(Copy, Clone, PartialEq)]
pub enum BranchTarget {
    Dynamic,
    Absolute(u32),
    Return(u32),
}