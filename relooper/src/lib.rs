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

// The Relooper accepts a map of block labels to the labels each block can branch to
pub fn reloop<L: RelooperLabel, S: BuildHasher>(blocks: HashMap<L, Vec<L>, S>, first_label: L) -> Box<ShapedBlock<L>> {
    let mut relooper = Relooper::new(blocks, first_label);
    relooper.process_loops();
    relooper.process_rejoined_branches();
    relooper.output(vec![relooper.nodes[&first_label]]).unwrap()
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
    pub immediate: Option<Box<ShapedBlock<L>>>,
    pub next: Option<Box<ShapedBlock<L>>>,
}

#[derive(Debug, PartialEq)]
pub struct LoopBlock<L: RelooperLabel> {
    pub loop_id: u16,
    pub inner: Box<ShapedBlock<L>>,
}

#[derive(Debug, PartialEq)]
pub struct MultipleBlock<L: RelooperLabel> {
    pub handled: Vec<HandledBlock<L>>,
}

#[derive(Debug, PartialEq)]
pub struct HandledBlock<L: RelooperLabel> {
    pub labels: Vec<L>,
    pub inner: Box<ShapedBlock<L>>,
}

// Branch modes
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BranchMode {
    Basic,
    LoopBreak(u16),
    LoopContinue(u16),
}

/* =======================
   Internal implementation
   ======================= */

type LoopId = u16;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Node<L> {
    Start,
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
    MergedBranch(L),
    Removed,
}

fn filter_edges<L>(edge: petgraph::graph::EdgeReference<Edge<L>>) -> bool {
    use Edge::*;
    match edge.weight() {
        Forward | ForwardMulti(_) | Next => true,
        _ => false,
    }
}

// Structure types, used in Relooper.move_undominated_edges_to_next
enum Structure {
    Loop(LoopId),
    Multiple,
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

        // Mark the start of the graph, to make identifying when the root node is a loop easier
        let start_node = graph.add_node(Node::Start);
        graph.add_edge(start_node, nodes[&root], Edge::Forward);

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

            // Get the strongly connected components
            let sccs = self.graph_sccs();
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
                //self.update_dominators();
                //self.move_undominated_edges_to_next(loop_node, loop_node, Structure::Loop(loop_id));
                // TODO: patch LoopBreakIntoMultiple edges into LoopBreak when there is only one Next node
            }

            if found_scc == false {
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
        let dominators = algo::dominators::simple_fast(&filtered_graph, self.nodes[&self.root]);
        let nodes = algo::toposort(&filtered_graph, None).unwrap();

        // Now in reverse order, go through the nodes, looking for those that have multiple incoming edges
        for &node in nodes.iter().rev() {
            let mut incoming_edges = Vec::default();
            for edge in self.graph.edges_directed(node, Incoming) {
                if let Edge::Forward = edge.weight() {
                    incoming_edges.push(edge.id());
                }
            }
            if incoming_edges.len() > 1 {
                // Add a Next edge to the dominator pointing to this node
                let dominator = dominators.immediate_dominator(node).unwrap();
                self.graph.add_edge(dominator, node, Edge::Next);
                // Patch the incoming edges
                for edge in incoming_edges {
                    self.graph[edge] = Edge::MergedBranch(self.get_basic_node_label(node));
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
            match node {
                Node::Basic(label) => {
                    let mut immediate_entries = Vec::default();
                    let mut next_entries = Vec::default();
                    for edge in self.graph.edges(node_id) {
                        match edge.weight() {
                            Edge::Forward => immediate_entries.push(edge.target()),
                            Edge::Next => next_entries.push(edge.target()),
                            _ => {},
                        };
                    }
                    return Some(Box::new(ShapedBlock::Simple(SimpleBlock {
                        label: *label,
                        immediate: self.output(immediate_entries),
                        next: self.output(next_entries),
                    })))
                },
                Node::Loop(loop_id) => {
                    let mut edges = self.graph.neighbors(node_id).detach();
                    let mut inner_entries = Vec::default();
                    let mut next_entries = Vec::new();
                    while let Some((edge, target)) = edges.next(&self.graph) {
                        match self.graph[edge] {
                            Edge::Forward => inner_entries.push(target),
                            Edge::Next => next_entries.push(target),
                            _ => {},
                        }
                    }
                    return Some(Box::new(ShapedBlock::Loop(LoopBlock {
                        loop_id: *loop_id,
                        inner: self.output(inner_entries).unwrap(),
                    })))
                },
                _ => unimplemented!(),
            };
        }

        // Multiples
        let mut handled = Vec::default();
        for entry in entries {
            handled.push(HandledBlock {
                labels: match self.graph[entry] {
                    Node::Basic(label) => vec![label],
                    Node::Loop(_) | Node::LoopMulti(_) => Vec::from_iter(self.graph.neighbors(entry).map(|n| self.get_basic_node_label(n))),
                    _ => unimplemented!(),
                },
                inner: self.output(vec![entry]).unwrap(),
            });
        }
        // Sort so that the tests will work
        handled.sort_by(|a, b| a.labels.cmp(&b.labels));
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

    // Given a node, find edges which are not dominated by it (loop or branch), and convert the edges to next edges on the structure head
    fn move_undominated_edges_to_next(&mut self, dominator: NodeIndex, structure_head: NodeIndex, mode: Structure) {
        // Prepare to manually walk through the graph
        let mut stack = vec![dominator];
        let mut discovered = self.graph.visit_map();

        while let Some(node) = stack.pop() {
            if discovered.visit(node) {
                let mut edges = self.graph.neighbors(node).detach();
                'edge_loop: while let Some((edge, target)) = edges.next(&self.graph) {
                    match self.graph[edge] {
                        Edge::Forward => {
                            let dominators = self.dominators.strict_dominators(target).unwrap();
                            for dom in dominators {
                                if dom == dominator {
                                    // This node is dominated by the structural dominator, so add it to the stack
                                    if !discovered.is_visited(&target) {
                                        stack.push(target);
                                    }
                                    continue 'edge_loop;
                                }
                            }
                            // Not dominated, so convert the edges
                            self.graph.update_edge(node, target, match mode {
                                Structure::Loop(loop_id) => Edge::LoopBreakIntoMultiple(loop_id),
                                Structure::Multiple => unimplemented!(),
                            });
                            // Add an edge to the structure head
                            self.graph.update_edge(structure_head, target, Edge::Next);
                        },
                        _ => {},
                    };
                }
            }
        }
    }

    fn update_dominators(&mut self) {
        // Filter the graph to ignore processed back edges
        let filtered_graph = EdgeFiltered::from_fn(&self.graph, filter_edges);
        self.dominators = algo::dominators::simple_fast(&filtered_graph, self.nodes[&self.root]);
    }
}