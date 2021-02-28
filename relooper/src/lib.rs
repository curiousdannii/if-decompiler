/*

Relooper library
================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

Based on the Relooper algorithm paper by Alon Zakai
https://github.com/emscripten-core/emscripten/blob/master/docs/paper.pdf

*/

#![forbid(unsafe_code)]

use core::hash::Hash;
use std::fmt::Display;

use fnv::{FnvHashMap, FnvHashSet};
use petgraph::{algo, graph, visit};
use graph::Graph;

// Common traits for labels
pub trait RelooperLabel: Copy + Display + Eq + Hash {}
impl<T> RelooperLabel for T
where T: Copy + Display + Eq + Hash {}

// Relooper accepts BasicBlocks as its input
pub struct BasicBlock<L: RelooperLabel, C> {
    pub label: L,
    pub code: Vec<C>,
    pub branches: FnvHashSet<L>,
}

// And returns a ShapedBlock tree
pub enum ShapedBlock<L: RelooperLabel> {
    Simple(SimpleBlock<L>),
    Loop(LoopBlock<L>),
    Multiple(MultipleBlock<L>),
    None,
}

pub struct SimpleBlock<L: RelooperLabel> {
    pub label: L,
    pub next: Box<ShapedBlock<L>>,
}

pub struct LoopBlock<L: RelooperLabel> {
    pub label: L,
    pub next: Box<ShapedBlock<L>>,
}

pub struct MultipleBlock<L: RelooperLabel> {
    pub label: L,
    pub next: Box<ShapedBlock<L>>,
}

// The Relooper algorithm
pub struct Relooper<'a, L: RelooperLabel, C> {
    block_labels: FnvHashMap<L, usize>,
    blocks: &'a Vec<BasicBlock<L, C>>,
    graph: Graph<L, ()>,
    nodes: FnvHashMap<L, graph::NodeIndex>,
}

impl<'a, L: RelooperLabel, C> Relooper<'a, L, C> {
    pub fn new(blocks: &'a Vec<BasicBlock<L, C>>) -> Relooper<L, C> {
        // Start by building a graph of blocks
        let mut graph: Graph<L, ()> = Graph::new();

        // Build a map of nodes
        let mut block_labels = FnvHashMap::default();
        let mut nodes = FnvHashMap::default();
        for (i, block) in blocks.iter().enumerate() {
            block_labels.insert(block.label, i);
            nodes.insert(block.label, graph.add_node(block.label));
        }

        // Add the edges
        for block in blocks {
            for branch in &block.branches {
                graph.add_edge(nodes[&block.label], nodes[&branch], ());
            }
        }

        Relooper {
            block_labels,
            blocks,
            graph,
            nodes,
        }
    }

    pub fn reloop(&self) -> ShapedBlock<L> {
        self.reloop_reduce(vec![self.blocks[0].label])
    }

    fn reloop_reduce(&self, entries: Vec<L>) -> ShapedBlock<L> {
        // If we have a single entry, and cannot return to it, create a simple block
        if entries.len() == 1 {
            let label = entries[0];
            let node = self.nodes[&label];
            if !is_node_in_cycle(&self.graph, node) {
                println!("simple block {}", label);
                let block = &self.blocks[self.block_labels[&label]];
                let mut new_entries = Vec::new();
                for branch in block.branches.iter() {
                    new_entries.push(*branch);
                }
                return ShapedBlock::Simple(SimpleBlock {
                    label,
                    next: Box::new(self.reloop_reduce(new_entries)),
                })
            }
        }

        ShapedBlock::None
    }
}

fn is_node_in_cycle<G>(graph: G, node: G::NodeId) -> bool where G: visit::IntoNeighbors + visit::Visitable {
    let mut space = algo::DfsSpace::new(&graph);
    for neighbour in graph.neighbors(node) {
        if algo::has_path_connecting(graph, neighbour, node, Some(&mut space)) {
            return true
        }
    }
    false
}