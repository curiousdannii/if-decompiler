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
use petgraph::algo;
use petgraph::visit::{EdgeFiltered, IntoNeighbors};

mod graph;
use graph::FilteredDfs;

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

fn filter_branch_modes(edge: petgraph::graph::EdgeReference<BranchMode>) -> bool {
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
            if !self.is_node_in_cycle(self.nodes[&label]) {
                let new_entries = self.blocks[&label].clone();
                return Some(Box::new(ShapedBlock::Simple(SimpleBlock {
                    label,
                    next: self.reloop_reduce(new_entries),
                })))
            }
        }

        // If we can return to all of the entries, create a loop block
        if self.do_all_entries_loop(&entries) {
            let loop_id = self.counter;
            self.counter += 1;
            let next_entries = self.make_loop(&entries, loop_id);
            return Some(Box::new(ShapedBlock::Loop(LoopBlock {
                loop_id,
                inner: self.reloop_reduce(entries).unwrap(),
                next: self.reloop_reduce(next_entries),
            })))
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

    fn do_all_entries_loop(&self, entries: &Vec<L>) -> bool {
        let filtered_graph = EdgeFiltered::from_fn(&self.graph, filter_branch_modes);
        let mut space = algo::DfsSpace::new(&filtered_graph);
        'entry_loop: for entry in entries {
            let entry_node = self.nodes[&entry];
            // Each entry must have one branch that loops back
            for neighbour in filtered_graph.neighbors(entry_node) {
                if algo::has_path_connecting(&filtered_graph, neighbour, entry_node, Some(&mut space)) {
                    continue 'entry_loop;
                }
            }
            return false;
        }
        true
    }

    fn make_loop(&mut self, entries: &Vec<L>, loop_id: u16) -> Vec<L> {
        use BranchMode::*;
        let mut space = algo::DfsSpace::new(&self.graph);
        let mut dfs = FilteredDfs::empty(&self.graph, |weight| weight == Basic);
        dfs.stack = entries.iter().map(|addr| self.nodes[addr]).collect();
        // Go through the graph, changing edges if they are branches back to an entry or if they exit the loop
        let mut next_entries = Vec::default();
        while let Some(node) = dfs.next(&self.graph) {
            'edges_loop: while let Some((edge, target)) = self.graph.neighbors(node).detach().next(&self.graph) {
                if self.graph[edge] != Basic {
                    continue 'edges_loop;
                }
                let target_addr = self.graph[target];
                // Branching back to an entry -> convert to a continue
                if entries.contains(&target_addr) {
                    self.graph[edge] = LoopContinue(loop_id);
                }
                // If the branch can't return to any entry, convert it to a break
                else {
                    for entry in entries {
                        if algo::has_path_connecting(&self.graph, target, self.nodes[&entry], Some(&mut space)) {
                            continue 'edges_loop;
                        }
                    }
                    self.graph[edge] = LoopBreak(loop_id);
                    next_entries.push(target_addr);
                }
            }
        }
        next_entries
    }
}