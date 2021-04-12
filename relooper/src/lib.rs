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
use petgraph::visit::{EdgeFiltered, IntoNeighbors, Visitable, VisitMap};

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

/* =======================
   Internal implementation
   ======================= */

type LoopId = u16;

#[derive(Debug)]
enum Node<L> {
    Basic(L),
    Loop(LoopId),
    LoopMulti(LoopId),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Edge<L> {
    Forward,
    ForwardMulti(L),
    Next,
    LoopBreak(LoopId),
    LoopBreakIntoMultiple((L, LoopId)),
    LoopContinue(LoopId),
    LoopContinueMulti((L, LoopId)),
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

                // If branching to a loop header, convert to a back edge
                for &node in &scc {
                    let mut edges = self.graph.neighbors(node).detach();
                    while let Some((edge, target)) = edges.next(&self.graph) {
                        if loop_headers.contains(&target) {
                            let target_label = match self.graph[target] {
                                Node::Basic(label) => label,
                                _ => panic!("Cannot replace an edge to a loop node"),
                            };
                            self.graph.add_edge(node, loop_node, if multi_loop { Edge::LoopContinueMulti((target_label, loop_id)) } else { Edge::LoopContinue(loop_id) });
                            // Not sure if it's safe to directly remove the edge here
                            self.graph[edge] = Edge::Removed;
                        }
                    };
                }
                // Fix edges which are branching outside the loop
                self.update_dominators();
                self.move_undominated_edges_to_next(loop_node, loop_node, Structure::Loop(loop_id));
                // TODO: patch LoopBreakIntoMultiple edges into LoopBreak when there is only one Next node
            }

            if found_scc == false {
                break;
            }
        }

        let filtered_graph = EdgeFiltered::from_fn(&self.graph, filter_edges);
        assert!(!algo::is_cyclic_directed(&filtered_graph), "Graph should not contain any cycles");
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
                            let target_label = match self.graph[target] {
                                Node::Basic(label) => label,
                                _ =>  panic!("Cannot replace an edge to a loop node"),
                            };
                            self.graph.update_edge(node, target, match mode {
                                Structure::Loop(loop_id) => Edge::LoopBreakIntoMultiple((target_label, loop_id)),
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
                    return (Some(Box::new(ShapedBlock::Loop(LoopBlock {
                        loop_id: *loop_id,
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