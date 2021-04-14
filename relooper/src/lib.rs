/*

Relooper library
================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

Inspired by the Cheerp Stackifier algorithm
https://medium.com/leaningtech/solving-the-structured-control-flow-problem-once-and-for-all-5123117b1ee2

And the Relooper algorithm paper by Alon Zakai
https://github.com/emscripten-core/emscripten/blob/master/docs/paper.pdf

*/

#![forbid(unsafe_code)]

use core::hash::{BuildHasher, Hash};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::iter::FromIterator;

use fnv::{FnvHashMap, FnvHashSet};
use petgraph::prelude::*;
use petgraph::algo;
use petgraph::visit::{EdgeFiltered, Visitable, VisitMap};

//mod graph;
//use graph::*;

#[cfg(test)]
mod tests;

// Common traits for labels
pub trait RelooperLabel: Copy + Debug + Display + Eq + Hash + Ord {}
impl<T> RelooperLabel for T
where T: Copy + Debug + Display + Eq + Hash + Ord {}

type LoopId = u16;

// The Relooper accepts a map of block labels to the labels each block can branch to
pub fn reloop<L: RelooperLabel, S: BuildHasher>(blocks: HashMap<L, Vec<L>, S>, first_label: L) -> Box<ShapedBlock<L>> {
    let mut relooper = Relooper::new(blocks, first_label);
    relooper.process_loops();
    relooper.process_rejoined_branches();
    relooper.output(vec![relooper.root]).unwrap()
}

// And returns a ShapedBlock tree
#[derive(Debug, PartialEq)]
pub enum ShapedBlock<L: RelooperLabel> {
    Simple(SimpleBlock<L>),
    Loop(LoopBlock<L>),
    LoopMulti(LoopMultiBlock<L>),
    Multiple(MultipleBlock<L>),
}

#[derive(Debug, PartialEq)]
pub struct SimpleBlock<L: RelooperLabel> {
    pub label: L,
    pub immediate: Option<Box<ShapedBlock<L>>>,
    pub next: Option<Box<ShapedBlock<L>>>,
    pub branches: FnvHashMap<L, BranchMode>,
}

// Branch modes
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BranchMode {
    LoopBreak(LoopId),
    LoopBreakIntoMultiple(LoopId),
    LoopContinue(LoopId),
    LoopContinueMulti(LoopId),
    MergedBranch,
    MergedBranchIntoMulti,
}

#[derive(Debug, PartialEq)]
pub struct LoopBlock<L: RelooperLabel> {
    pub loop_id: u16,
    pub inner: Box<ShapedBlock<L>>,
}
#[derive(Debug, PartialEq)]
pub struct LoopMultiBlock<L: RelooperLabel> {
    pub loop_id: u16,
    pub handled: Vec<HandledBlock<L>>,
    pub next: Option<Box<ShapedBlock<L>>>,
}

#[derive(Debug, PartialEq)]
pub struct MultipleBlock<L: RelooperLabel> {
    // It would be nicer to use a Hashmap here, but if the graph ever has triple branches it's possible you'd have a Multiple going into a LoopMulti, so we need a Vec of handled labels
    pub handled: Vec<HandledBlock<L>>,
}

#[derive(Debug, PartialEq)]
pub struct HandledBlock<L: RelooperLabel> {
    pub labels: Vec<L>,
    pub inner: Box<ShapedBlock<L>>,
}

/* =======================
   Internal implementation
   ======================= */

#[derive(Clone, Copy, Debug, PartialEq)]
enum Node<L> {
    Root,
    Basic(L),
    Loop(LoopId),
    LoopMulti(LoopId),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Edge<L> {
    // Edges that form the acyclic CFG
    Forward,
    ForwardMulti(L),
    Next,
    // Edges that will be filtered out when traversing the graph
    LoopBreak(LoopId),
    LoopBreakIntoMultiple(LoopId),
    LoopContinue(LoopId),
    LoopContinueMulti(LoopId),
    MergedBranch,
    MergedBranchIntoMulti,
    Removed,
}

fn filter_edges<L>(edge: petgraph::graph::EdgeReference<Edge<L>>) -> bool {
    use Edge::*;
    match edge.weight() {
        Forward | ForwardMulti(_) | Next => true,
        _ => false,
    }
}

// The Relooper algorithm
struct Relooper<L: RelooperLabel> {
    counter: u16,
    graph: Graph<Node<L>, Edge<L>>,
    root: NodeIndex,
}

impl<L: RelooperLabel> Relooper<L> {
    fn new<S>(blocks: HashMap<L, Vec<L>, S>, root_label: L) -> Relooper<L>
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

        // Add a root node to the graph, in order to handle when the first label is a loop
        let root = graph.add_node(Node::Root);
        graph.add_edge(root, nodes[&root_label], Edge::Forward);

        Relooper {
            counter: 0,
            graph,
            root,
        }
    }

    // Process loops by adding loop nodes and converting back edges
    fn process_loops(&mut self) {
        // Loop until we have no more SCCs
        loop {
            let mut found_loop = false;

            // Get the strongly connected components
            let sccs = self.graph_sccs();
            for scc in sccs {
                if scc.len() == 1 {
                    // Test for self-loops
                    let node = scc[0];
                    let mut found_self_loop = false;
                    for edge in self.graph.edges(node) {
                        if let Edge::Forward = edge.weight() {
                            if edge.target() == node {
                                found_self_loop = true;
                                break;
                            }
                        }
                    }
                    if !found_self_loop {
                        continue;
                    }
                }
                found_loop = true;

                // Determine whether this is a multi-loop or not
                // Get all incoming edges and find the loop headers
                let mut edges = Vec::default();
                let mut loop_headers = FnvHashSet::default();
                let mut loop_at_root = false;
                for &node in &scc {
                    for edge in self.graph.edges_directed(node, Incoming) {
                        if !scc.contains(&edge.source()) {
                            loop_headers.insert(edge.target());
                            edges.push((edge.id(), edge.source(), edge.target()));
                            if edge.target() == self.root {
                                loop_at_root = true;
                            }
                        }
                    }
                }
                let multi_loop = loop_headers.len() > 1;

                // Add the new node
                let loop_id = self.counter;
                self.counter += 1;
                let loop_node = self.graph.add_node(if multi_loop { Node::LoopMulti(loop_id) } else { Node::Loop(loop_id) });

                // Replace the incoming edges
                for edge in edges {
                    let target_label = self.get_basic_node_label(edge.2);
                    self.graph.add_edge(edge.1, loop_node, if multi_loop { Edge::ForwardMulti(target_label) } else { Edge::Forward });
                    // Cannot remove edges without potentially breaking other edge indexes, so mark them as removed for now
                    self.graph[edge.0] = Edge::Removed;
                }
                // Now add edges from the new node to the loop header(s)
                for &header in &loop_headers {
                    self.graph.add_edge(loop_node, header, Edge::Forward);
                }

                // If branching to a loop header, convert to a back edge
                for &node in &scc {
                    let mut edges = self.graph.neighbors(node).detach();
                    while let Some((edge, target)) = edges.next(&self.graph) {
                        if loop_headers.contains(&target) {
                            self.graph[edge] = if multi_loop { Edge::LoopContinueMulti(loop_id) } else { Edge::LoopContinue(loop_id) };
                        }
                    };
                }

                // Fix edges which are branching outside the loop
                // But we don't need to if the loop_node is at the very top
                if loop_at_root {
                    continue;
                }

                let filtered_graph = EdgeFiltered::from_fn(&self.graph, filter_edges);
                let dominators = algo::dominators::simple_fast(&filtered_graph, self.root);
                let structure_head = if multi_loop { loop_node } else { dominators.immediate_dominator(loop_node).unwrap() };

                // Walk through the graph manually
                let mut stack = vec![loop_node];
                let mut discovered = self.graph.visit_map();

                while let Some(node) = stack.pop() {
                    if discovered.visit(node) {
                        let mut edges = self.graph.neighbors(node).detach();
                        'edge_loop: while let Some((edge, target)) = edges.next(&self.graph) {
                            if let Edge::Forward = self.graph[edge] {
                                let dominators = dominators.strict_dominators(target).unwrap();
                                for dom in dominators {
                                    if dom == loop_node {
                                        // This node is dominated by the structural dominator, so add it to the stack
                                        if !discovered.is_visited(&target) {
                                            stack.push(target);
                                        }
                                        continue 'edge_loop;
                                    }
                                }
                                // Not dominated, so convert the edges
                                self.graph.update_edge(node, target, Edge::LoopBreak(loop_id));
                                // Add an edge to the structure head
                                self.graph.update_edge(structure_head, target, Edge::Next);
                            }
                        }
                    }
                }
            }

            if found_loop == false {
                break;
            }
        }

        let filtered_graph = EdgeFiltered::from_fn(&self.graph, filter_edges);
        assert!(!algo::is_cyclic_directed(&filtered_graph), "Graph should not contain any cycles");
    }

    // Handle branches that merge back together
    fn process_rejoined_branches(&mut self) {
        // Get the list of nodes in topological order and the dominators list
        let filtered_graph = EdgeFiltered::from_fn(&self.graph, filter_edges);
        let dominators = algo::dominators::simple_fast(&filtered_graph, self.root);
        let nodes = algo::toposort(&filtered_graph, None).unwrap();

        // Now in reverse order, go through the nodes, looking for those that have multiple incoming edges
        for &node in nodes.iter().rev() {
            let mut incoming_edges = Vec::default();
            for edge in self.graph.edges_directed(node, Incoming) {
                match edge.weight() {
                    Edge::Forward | Edge::LoopBreak(_) | Edge::LoopBreakIntoMultiple(_) => incoming_edges.push(edge.id()),
                    _ => {},
                }
            }
            if incoming_edges.len() > 1 {
                let dominator = dominators.immediate_dominator(node).unwrap();

                // First check if the dominator already has a different next node
                let mut other_next = false;
                'outer_loop: for edge in self.graph.edges(dominator) {
                    if let Edge::Next = edge.weight() {
                        if edge.target() != node {
                            other_next = true;
                            // Now check if the next node has not already been processed
                            for incoming_edge in self.graph.edges_directed(edge.target(), Incoming) {
                                match incoming_edge.weight() {
                                    Edge::LoopBreak(_) | Edge::MergedBranch => {
                                        // If so, patch the incoming edges to the first next
                                        let mut edges = self.graph.neighbors_directed(edge.target(), Incoming).detach();
                                        while let Some((edge, _)) = edges.next(&self.graph) {
                                            match self.graph[edge] {
                                                Edge::LoopBreak(loop_id) => {
                                                    self.graph[edge] = Edge::LoopBreakIntoMultiple(loop_id);
                                                },
                                                Edge::MergedBranch => {
                                                    self.graph[edge] = Edge::MergedBranchIntoMulti;
                                                },
                                                _ => {},
                                            };
                                        }
                                        break 'outer_loop;
                                    },
                                    _ => {},
                                }
                            }
                            break;
                        }
                    }
                }

                // Add a Next edge to the dominator pointing to this node
                self.graph.add_edge(dominator, node, Edge::Next);
                // Patch the incoming edges
                for edge in incoming_edges {
                    match self.graph[edge] {
                        Edge::Forward => self.graph[edge] = if other_next { Edge::MergedBranchIntoMulti } else { Edge::MergedBranch },
                        Edge::LoopBreak(loop_id) => self.graph[edge] = if other_next { Edge::LoopBreakIntoMultiple(loop_id) } else { Edge::LoopBreak(loop_id) },
                        _ => {},
                    };
                }
            }
        }
    }

    // Output the graph as blocks
    fn output(&self, entries: Vec<NodeIndex>) -> Option<Box<ShapedBlock<L>>> {
        if entries.len() == 0 {
            return None
        }

        // If we have one entry, then return the appropriate block
        if entries.len() == 1 {
            let node_id = entries[0];
            let node = &self.graph[node_id];
            let mut all_entries = FnvHashSet::default();
            let mut immediate_entries = FnvHashSet::default();
            let mut next_entries = FnvHashSet::default();
            let mut outgoing_branches = FnvHashMap::default();
            for edge in self.graph.edges(node_id) {
                if let Edge::Removed = edge.weight() {
                    continue;
                }
                let target = edge.target();
                all_entries.insert(target);
                let mut add_branch = |target, branch| outgoing_branches.insert(self.get_basic_node_label(target), branch);
                match edge.weight() {
                    Edge::Forward | Edge::ForwardMulti(_) => { immediate_entries.insert(target); },
                    Edge::Next => { next_entries.insert(target); },
                    Edge::LoopBreak(loop_id) => { add_branch(target, BranchMode::LoopBreak(*loop_id)); },
                    Edge::LoopBreakIntoMultiple(loop_id) => { add_branch(target, BranchMode::LoopBreakIntoMultiple(*loop_id)); },
                    Edge::LoopContinue(loop_id) => { add_branch(target, BranchMode::LoopContinue(*loop_id)); },
                    Edge::LoopContinueMulti(loop_id) => { add_branch(target, BranchMode::LoopContinueMulti(*loop_id)); },
                    Edge::MergedBranch => { add_branch(target, BranchMode::MergedBranch); },
                    Edge::MergedBranchIntoMulti => { add_branch(target, BranchMode::MergedBranchIntoMulti); },
                    _ => {},
                };
            }
            // Dedup the ForwardMulti edges
            let immediate_entries = Vec::from_iter(immediate_entries);
            let next_entries = Vec::from_iter(next_entries);

            return match node {
                Node::Root => {
                    assert_eq!(next_entries.len(), 0, "Root node should have no next entries");
                    self.output(immediate_entries)
                },
                Node::Basic(label) => {
                    Some(Box::new(ShapedBlock::Simple(SimpleBlock {
                        label: *label,
                        // If we originally branched here, return a MultipleBlock even if there's now only one immediate entry
                        immediate: if all_entries.len() > 1 && immediate_entries.len() > 0 {
                            let handled = self.output_multiple_handled(immediate_entries);
                            Some(Box::new(ShapedBlock::Multiple(MultipleBlock {
                                handled,
                            })))
                        }
                        else {
                            self.output(immediate_entries)
                        },
                        next: self.output(next_entries),
                        branches: outgoing_branches,
                    })))
                },
                Node::Loop(loop_id) => {
                    assert_eq!(next_entries.len(), 0, "Loop nodes should have no next entries");
                    Some(Box::new(ShapedBlock::Loop(LoopBlock {
                        loop_id: *loop_id,
                        inner: self.output(immediate_entries).unwrap(),
                    })))
                },
                Node::LoopMulti(loop_id) => {
                    let handled = self.output_multiple_handled(immediate_entries);
                    Some(Box::new(ShapedBlock::LoopMulti(LoopMultiBlock {
                        loop_id: *loop_id,
                        handled,
                        next: self.output(next_entries),
                    })))
                },
            }
        }

        // Multiples
        let handled = self.output_multiple_handled(entries);
        Some(Box::new(ShapedBlock::Multiple(MultipleBlock {
            handled,
        })))
    }

    fn get_basic_node_label(&self, id: NodeIndex) -> L {
        match self.graph[id] {
            Node::Basic(label) => label,
            _ => panic!("Cannot get label of loop node"),
        }
    }

    // Return the SCCs over a filtered graph
    fn graph_sccs(&self) -> Vec<Vec<NodeIndex>> {
        // Filter the graph to ignore processed back edges
        let filtered_graph = EdgeFiltered::from_fn(&self.graph, filter_edges);
        algo::kosaraju_scc(&filtered_graph)
    }

    fn output_multiple_handled(&self, entries: Vec<NodeIndex>) -> Vec<HandledBlock<L>> {
        let mut handled = Vec::default();
        for entry in entries {
            handled.push(HandledBlock {
                labels: match self.graph[entry] {
                    Node::Basic(label) => vec![label],
                    Node::Loop(_) | Node::LoopMulti(_) => {
                        let mut labels = Vec::default();
                        let mut edges = self.graph.neighbors(entry).detach();
                        while let Some((edge, target)) = edges.next(&self.graph) {
                            if let Edge::Forward = self.graph[edge] {
                                labels.push(self.get_basic_node_label(target));
                            }
                        }
                        labels.sort();
                        labels
                    },
                    _ => unimplemented!(),
                },
                inner: self.output(vec![entry]).unwrap(),
            });
        }
        // Sort so that the tests will work
        handled.sort_by(|a, b| a.labels.cmp(&b.labels));
        handled
    }
}