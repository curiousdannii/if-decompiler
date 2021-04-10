/*

Relooper library
================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

Inspired by the Relooper algorithm paper by Alon Zakai
https://github.com/emscripten-core/emscripten/blob/master/docs/paper.pdf

And this article about the Cheerp Stackifier algorithm
https://medium.com/leaningtech/solving-the-structured-control-flow-problem-once-and-for-all-5123117b1ee2

*/

#![forbid(unsafe_code)]

use core::hash::{BuildHasher, Hash};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::fmt::{Debug, Display};

use fnv::{FnvHashMap, FnvHashSet};
use petgraph::prelude::*;
use petgraph::algo;
use petgraph::visit::{EdgeFiltered, IntoNeighbors};

//mod graph;
//use graph::*;

#[cfg(test)]
mod tests;

// Common traits for labels
pub trait RelooperLabel: Copy + Debug + Display + Eq + Hash {}
impl<T> RelooperLabel for T
where T: Copy + Debug + Display + Eq + Hash {}

// The Relooper accepts a map of block labels to the labels each block can branch to
pub fn reloop<L: RelooperLabel, S: BuildHasher>(blocks: HashMap<L, Vec<L>, S>, first_label: L) -> Box<ShapedBlock<L>> {
    let mut relooper = Relooper::new(blocks, first_label);
    relooper.process_loops();
    relooper.output().unwrap()
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
    pub handled: FnvHashMap<L, Box<ShapedBlock<L>>>,
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
struct LoopData {
    id: LoopId,
    next: FnvHashSet<NodeIndex>,
}

#[derive(Debug)]
enum Node<L> {
    Basic(L),
    Loop(LoopData),
    LoopMulti(LoopData),
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
    dominators: algo::dominators::Dominators<NodeIndex>,
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
            dominators: algo::dominators::simple_fast(&graph, nodes[&root]),
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

            // Re-calculate dominators for determining loop breaks, etc
            self.dominators = algo::dominators::simple_fast(&filtered_graph, self.nodes[&self.root]);

            // Run the SCC algorithm
            let sccs = algo::kosaraju_scc(&filtered_graph);
            for scc in sccs {
                if scc.len() == 1 {
                    continue;
                }
                found_scc = true;

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
                let loop_node = self.graph.add_node(if multi_loop { Node::LoopMulti(loop_data) } else { Node::Loop(loop_data) });

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
                            if let Some(dominator) = self.dominators.immediate_dominator(target) {
                                if dominator != node {
                                    self.graph[edge] = Edge::LoopBreak(loop_id);
                                    let loop_data = &mut self.graph[loop_node];
                                    match loop_data {
                                        Node::Basic(_) => unreachable!(),
                                        Node::Loop(data) | Node::LoopMulti(data) => {
                                            data.next.insert(target);
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

    fn output(&self) -> Option<Box<ShapedBlock<L>>> {
        self.reduce(vec![self.nodes[&self.root]])
    }

    fn reduce(&self, entries: Vec<NodeIndex>) -> Option<Box<ShapedBlock<L>>> {
        let result = self.reduce_with_next(entries);
        assert!(result.1.is_none(), "No dangling next entries");
        result.0
    }

    fn reduce_with_next(&self, entries: Vec<NodeIndex>) -> (Option<Box<ShapedBlock<L>>>, Option<NodeIndex>) {
        if entries.len() == 0 {
            return (None, None)
        }

        let filtered_graph = EdgeFiltered::from_fn(&self.graph, filter_edges);

        // If we have one entry, then return the appropriate block
        if entries.len() == 1 {
            let node_id = entries[0];
            let node = &self.graph[node_id];
            return match node {
                Node::Basic(label) => {
                    let mut next = None;
                    let next_entries = Vec::from_iter(filtered_graph.neighbors(node_id));
                    if next_entries.len() == 1 {
                        let next_node = next_entries[0];
                        if let Some(dominator) = self.dominators.immediate_dominator(next_node) {
                            if dominator != node_id {
                                next = Some(next_node);
                            }
                        }
                    }
                    return (Some(Box::new(ShapedBlock::Simple(SimpleBlock {
                        label: *label,
                        next: if next.is_none() { self.reduce(next_entries) } else { None },
                    }))), next)
                },
                Node::Loop(data) => {
                    let inner_entries = Vec::from_iter(filtered_graph.neighbors(node_id));
                    let mut next_entries = Vec::new();
                    for entry in &data.next {
                        next_entries.push(*entry);
                    }
                    return (Some(Box::new(ShapedBlock::Loop(LoopBlock {
                        loop_id: data.id,
                        inner: self.reduce(inner_entries).unwrap(),
                        next: self.reduce(next_entries),
                    }))), None)
                },
                _ => { (None, None) },
            }
        }

        // Handle multiple entries
        let mut handled = FnvHashMap::default();
        let mut next_entries = FnvHashSet::default();
        for entry in entries {
            let node = &self.graph[entry];
            let label = match node {
                Node::Basic(label) => *label,
                _ => panic!("Non-basic nodes in multiple"),
            };
            let result = self.reduce_with_next(vec![entry]);
            if let Some(handled_node) = result.0 {
                handled.insert(label, handled_node);
            }
            if let Some(next) = result.1 {
                next_entries.insert(next);
            }
        }
        (Some(Box::new(ShapedBlock::Multiple(MultipleBlock {
            handled,
            next: self.reduce(Vec::from_iter(next_entries)),
        }))), None)
    }
}