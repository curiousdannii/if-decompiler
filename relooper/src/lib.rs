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

use core::hash::{BuildHasher, Hash};
use std::collections::HashMap;
use std::fmt::Display;

use fnv::FnvHashMap;
use petgraph::prelude::*;
use petgraph::{algo, graph};
use petgraph::visit::{EdgeFiltered, IntoNeighbors};

#[cfg(test)]
mod tests;

// Common traits for labels
pub trait RelooperLabel: Copy + Display + Eq + Hash {}
impl<T> RelooperLabel for T
where T: Copy + Display + Eq + Hash {}

// The Relooper accepts a map of block labels to the labels each block can branch to
pub fn reloop<L: RelooperLabel, S: BuildHasher>(blocks: HashMap<L, Vec<L>, S>, first_label: L) -> Box<ShapedBlock<L>> {
    let mut relooper = Relooper::new(blocks);
    relooper.reloop_reduce(vec![first_label]).unwrap()
}

// And returns a ShapedBlock tree
#[derive(Debug, PartialEq)]
pub enum ShapedBlock<L: RelooperLabel> {
    Simple(SimpleBlock<L>),
    Loop(LoopBlock<L>),
    Multiple(MultipleBlock<L>),
}

#[derive(Debug, PartialEq)]
pub struct SimpleBlock<L: RelooperLabel> {
    pub label: L,
    pub next: Option<Box<ShapedBlock<L>>>,
}

#[derive(Debug, PartialEq)]
pub struct LoopBlock<L: RelooperLabel> {
    pub loop_id: u16,
    pub inner: Box<ShapedBlock<L>>,
    pub next: Option<Box<ShapedBlock<L>>>,
}

#[derive(Debug, PartialEq)]
pub struct MultipleBlock<L: RelooperLabel> {
    pub label: L,
    pub next: Option<Box<ShapedBlock<L>>>,
}

// Branch modes
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BranchMode {
    Basic,
    LoopBreak(u16),
    LoopContinue(u16),
}

fn filter_branch_modes(edge: graph::EdgeReference<BranchMode>) -> bool {
    match edge.weight() {
        BranchMode::Basic => true,
        _ => false,
    }
}

// The Relooper algorithm
struct Relooper<L: RelooperLabel, S: BuildHasher> {
    blocks: HashMap<L, Vec<L>, S>,
    counter: u16,
    graph: Graph<L, BranchMode>,
    nodes: FnvHashMap<L, NodeIndex>,
}

impl<L: RelooperLabel, S: BuildHasher> Relooper<L, S> {
    fn new(blocks: HashMap<L, Vec<L>, S>) -> Relooper<L, S> {
        let mut graph = Graph::new();
        let mut nodes = FnvHashMap::default();

        // Add nodes for each block
        for label in blocks.keys() {
            nodes.insert(*label, graph.add_node(*label));
        }

        // Add the edges
        for (label, branches) in &blocks {
            for branch in branches {
                graph.add_edge(nodes[&label], nodes[&branch], BranchMode::Basic);
            }
        }

        Relooper {
            blocks,
            counter: 0,
            graph,
            nodes,
        }
    }

    fn reloop_reduce(&mut self, entries: Vec<L>) -> Option<Box<ShapedBlock<L>>> {
        // If we have a single entry, and cannot return to it, create a simple block
        if entries.len() == 1 {
            let label = entries[0];
            let node = self.nodes[&label];
            if !self.is_node_in_cycle(node) {
                println!("simple block {}", label);
                let mut new_entries = Vec::new();
                for branch in &self.blocks[&label] {
                    new_entries.push(*branch);
                }
                return Some(Box::new(ShapedBlock::Simple(SimpleBlock {
                    label,
                    next: self.reloop_reduce(new_entries),
                })))
            }
        }

        None
    }

    fn is_node_in_cycle(&self, node: NodeIndex) -> bool {
        let filtered_graph = EdgeFiltered::from_fn(&self.graph, filter_branch_modes);
        let mut space = algo::DfsSpace::new(&filtered_graph);
        for neighbour in filtered_graph.neighbors(node) {
            if algo::has_path_connecting(&filtered_graph, neighbour, node, Some(&mut space)) {
                return true
            }
        }
        false
    }
}