/*

Relooper library
================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

Based on the Relooper algorithm paper by Alon Zakai
https://github.com/emscripten-core/emscripten/blob/master/docs/paper.pdf

And this article about the Cheerp Stackifier algorithm
https://medium.com/leaningtech/solving-the-structured-control-flow-problem-once-and-for-all-5123117b1ee2

*/

#![forbid(unsafe_code)]

use core::hash::{BuildHasher, Hash};
use std::collections::HashMap;
use std::fmt::{Debug, Display};

use fnv::{FnvHashMap, FnvHashSet};
use petgraph::prelude::*;
use petgraph::algo;
use petgraph::visit::{EdgeFiltered, IntoNeighbors};

mod graph;
use graph::*;

#[cfg(test)]
mod tests;

// Common traits for labels
pub trait RelooperLabel: Copy + Debug + Display + Eq + Hash {}
impl<T> RelooperLabel for T
where T: Copy + Debug + Display + Eq + Hash {}

// The Relooper accepts a map of block labels to the labels each block can branch to
pub fn reloop<L: RelooperLabel, S: BuildHasher>(blocks: HashMap<L, Vec<L>, S>, first_label: L) {
    let mut relooper = Relooper::new(blocks, first_label);
    relooper.process_loops();
    //relooper.reloop_reduce(vec![first_label]).unwrap()
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

// Internal types
type LoopId = u16;

#[derive(Debug)]
struct LoopData<L> {
    id: LoopId,
    next: FnvHashSet<L>,
}

#[derive(Debug)]
enum Node<L> {
    Basic(L),
    Loop(LoopData<L>),
    LoopMulti(LoopData<L>),
}

#[derive(Debug)]
enum Edge<L> {
    Forward,
    ForwardMulti(L),
    LoopBreak(LoopId),
    Back(LoopId),
    BackMulti((L, LoopId)),
    Removed,
}

fn filter_edges<L>(edge: petgraph::graph::EdgeReference<Edge<L>>) -> bool {
    use Edge::*;
    match edge.weight() {
        Forward | ForwardMulti(_) | LoopBreak(_) => true,
        _ => false,
    }
}

// The Relooper algorithm
struct Relooper<L: RelooperLabel> {
    counter: u16,
    graph: Graph<Node<L>, Edge<L>>,
    nodes: FnvHashMap<L, NodeIndex>,
    root: L,
}

impl<L: RelooperLabel> Relooper<L> {
    fn new<S>(blocks: HashMap<L, Vec<L>, S>, root: L) -> Relooper<L>
    where S: BuildHasher
    {
        let mut graph = Graph::new();
        let mut nodes = FnvHashMap::default();

        // Add nodes for each block
        for label in blocks.keys() {
            nodes.insert(*label, graph.add_node(Node::Basic(*label)));
        }

        // Add the edges
        for (label, branches) in &blocks {
            for branch in branches {
                graph.add_edge(nodes[&label], nodes[&branch], Edge::Forward);
            }
        }

        Relooper {
            counter: 0,
            graph,
            nodes,
            root,
        }
    }

    // Process loops by adding loop nodes and converting back edges
    fn process_loops(&mut self) {
        // Loop until we have no more SCCs
        loop {
            let mut found_scc = false;

            // Filter the graph to ignore processed back edges
            let filtered_graph = EdgeFiltered::from_fn(&self.graph, filter_edges);

            // Calculate dominators for determining loop breaks later on
            let dominators = algo::dominators::simple_fast(&filtered_graph, self.nodes[&self.root]);

            // Run the SCC algorithm
            let sccs = algo::kosaraju_scc(&filtered_graph);
            for scc in sccs {
                if scc.len() == 1 {
                    continue;
                }
                found_scc = true;
                println!("{:?}", scc);

                // Determine whether this is a multi-loop or not
                // Get all incoming edges and find the loop headers
                let mut edges = Vec::default();
                let mut loop_headers = FnvHashSet::default();
                for &node in &scc {
                    for edge in self.graph.edges_directed(node, Incoming) {
                        if !scc.contains(&edge.source()) {
                            loop_headers.insert(edge.target());
                            edges.push((edge.id(), edge.source(), edge.target()));
                        }
                    }
                }
                let multi_loop = loop_headers.len() > 1;

                // Add the new node
                let loop_id = self.counter;
                self.counter += 1;
                let loop_data = LoopData {
                    id: loop_id,
                    next: FnvHashSet::default(),
                };
                let loop_node = self.graph.add_node(if multi_loop { Node::Loop(loop_data) } else { Node::LoopMulti(loop_data) });

                // Replace the incoming edges
                for edge in edges {
                    let target_label = match self.graph[edge.2] {
                        Node::Basic(label) => label,
                        _ => panic!("Cannot replace an edge to a loop node"),
                    };
                    self.graph.add_edge(edge.1, loop_node, if multi_loop { Edge::ForwardMulti(target_label) } else { Edge::Forward });
                    // Cannot remove edges without potentially breaking other edge indexes, so mark them as removed for now
                    self.graph[edge.0] = Edge::Removed;
                }
                // Now add edges from the new node to the loop header(s)
                for &header in &loop_headers {
                    self.graph.add_edge(loop_node, header, Edge::Forward);
                }

                // Now replace the outgoing edges
                for &node in &scc {
                    let mut edges = self.graph.neighbors(node).detach();
                    while let Some((edge, target)) = edges.next(&self.graph) {
                        // If branching to a loop header, convert to a back edge
                        if loop_headers.contains(&target) {
                            let target_label = match self.graph[target] {
                                Node::Basic(label) => label,
                                _ => panic!("Cannot replace an edge to a loop node"),
                            };
                            self.graph.add_edge(node, loop_node, if multi_loop { Edge::BackMulti((target_label, loop_id)) } else { Edge::Back(loop_id) });
                            // Not sure if it's safe to directly remove the edge here
                            self.graph[edge] = Edge::Removed;
                        }
                        // Otherwise if branching outside the SCC, convert to a Break if not dominated
                        // But this won't detect non-dominated descendent nodes?
                        else if !scc.contains(&target) {
                            if let Some(dominator) = dominators.immediate_dominator(target) {
                                if dominator != node {
                                    self.graph[edge] = Edge::LoopBreak(loop_id);
                                    let target_label = match self.graph[target] {
                                        Node::Basic(label) => label,
                                        _ => panic!("Cannot replace an edge to a loop node"),
                                    };
                                    let loop_data = &mut self.graph[loop_node];
                                    match loop_data {
                                        Node::Basic(_) => unreachable!(),
                                        Node::Loop(data) | Node::LoopMulti(data) => {
                                            data.next.insert(target_label);
                                        },
                                    };
                                }
                            }
                        }
                    };
                }
            }

            if found_scc == false {
                break;
            }
        }

        let filtered_graph = EdgeFiltered::from_fn(&self.graph, filter_edges);
        assert!(!algo::is_cyclic_directed(&filtered_graph), "Graph should not contain any cycles");
    }

    // Implement the Relooper algorithm found on pages 9-10 of the Relooper paper
    /*fn reloop_reduce(&mut self, entries: Vec<L>) -> Option<Box<ShapedBlock<L>>> {
        if entries.len() == 0 {
            return None
        }

        // If we have a single entry, and cannot return to it, create a simple block
        if entries.len() == 1 {
            let label = entries[0];
            let node = self.nodes[&label];
            if !self.is_node_in_cycle(node) {
                let mut new_entries = Vec::default();
                let filtered_graph = EdgeFiltered::from_fn(&self.graph, filter_branch_modes);
                for neighbour in filtered_graph.neighbors(node) {
                    new_entries.push(self.graph[neighbour]);
                }
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

        // If we have more than one entry, try to create a multiple block
        None
        /*if entries.len() > 1 {
            let (multiple_entries, next_entries) = self.make_multiple(&entries);
            if multiple_entries.len() > 0 {

            }
        }

        // If we couldn't make a multiple block, make a loop block
        let loop_id = self.counter;
        self.counter += 1;
        let next_entries = self.make_loop(&entries, loop_id);
        return Some(Box::new(ShapedBlock::Loop(LoopBlock {
            loop_id,
            inner: self.reloop_reduce(entries).unwrap(),
            next: self.reloop_reduce(next_entries),
        })))*/
    }*/

    /*fn is_node_in_cycle(&self, node: NodeIndex) -> bool {
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
            let mut edges = self.graph.neighbors(node).detach();
            'edges_loop: while let Some((edge, target)) = edges.next(&self.graph) {
                // Filter out branches that have already been transformed
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

    // Try to make a multiple block, but finding which entries have labels that can't be reached by any other entry
    fn make_multiple(&mut self, entries: &Vec<L>) -> (Vec<L>, Vec<L>) {
        let mut multiple_entries = Vec::default();
        let mut next_entries = Vec::default();
        let filtered_graph = EdgeFiltered::from_fn(&self.graph, filter_branch_modes);
        let mut dfs = Dfs::empty(&filtered_graph);
        let mut space = algo::DfsSpace::new(&self.graph);

        // For each entry, see if it has labels which can't be reached by any other entry
        for entry in entries {
            dfs.reset(&filtered_graph);
            dfs.move_to(self.nodes[&entry]);
            
        }

        (multiple_entries, next_entries)
    }*/
}