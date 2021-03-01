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
use petgraph::{algo, graph, visit};

// Common traits for labels
pub trait RelooperLabel: Copy + Display + Eq + Hash {}
impl<T> RelooperLabel for T
where T: Copy + Display + Eq + Hash {}

// The Relooper accepts a map of block labels to the labels each block can branch to
pub fn reloop<L: RelooperLabel, S: BuildHasher>(blocks: HashMap<L, Vec<L>, S>, first_label: L) -> ShapedBlock<L> {
    let relooper = Relooper::new(blocks);
    relooper.reloop_reduce(vec![first_label])
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
struct Relooper<L: RelooperLabel, S: BuildHasher> {
    blocks: HashMap<L, Vec<L>, S>,
    graph: graph::Graph<L, ()>,
    nodes: FnvHashMap<L, graph::NodeIndex>,
}

impl<L: RelooperLabel, S: BuildHasher> Relooper<L, S> {
    fn new(blocks: HashMap<L, Vec<L>, S>) -> Relooper<L, S> {
        let mut graph = graph::Graph::new();
        let mut nodes = FnvHashMap::default();

        // Add nodes for each block
        for label in blocks.keys() {
            nodes.insert(*label, graph.add_node(*label));
        }

        // Add the edges
        for (label, branches) in &blocks {
            for branch in branches {
                graph.add_edge(nodes[&label], nodes[&branch], ());
            }
        }

        Relooper {
            blocks,
            graph,
            nodes,
        }
    }

    fn reloop_reduce(&self, entries: Vec<L>) -> ShapedBlock<L> {
        // If we have a single entry, and cannot return to it, create a simple block
        if entries.len() == 1 {
            let label = entries[0];
            let node = self.nodes[&label];
            if !is_node_in_cycle(&self.graph, node) {
                println!("simple block {}", label);
                let mut new_entries = Vec::new();
                for branch in &self.blocks[&label] {
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