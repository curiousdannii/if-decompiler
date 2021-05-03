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

use core::hash::Hash;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::iter::FromIterator;

use fnv::{FnvHashMap, FnvHashSet};
use petgraph::prelude::*;
use petgraph::algo;
use petgraph::visit::{EdgeFiltered, Visitable, VisitMap};

#[cfg(test)]
mod tests;

// Common traits for labels
pub trait RelooperLabel: Copy + Debug + Eq + Hash + Ord {}
impl<T> RelooperLabel for T
where T: Copy + Debug + Eq + Hash + Ord {}

type LoopId = u16;

// The Relooper accepts a map of block labels to the labels each block can branch to
pub fn reloop<L: RelooperLabel>(blocks: BTreeMap<L, Vec<L>>, first_label: L) -> Box<ShapedBlock<L>> {
    let mut relooper = Relooper::new(blocks, first_label);
    relooper.process_loops();
    relooper.process_rejoined_branches();
    relooper.output(vec![relooper.graph_root]).unwrap()
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
    pub branches: FnvHashMap<L, BranchMode>,
    pub next: Option<Box<ShapedBlock<L>>>,
}

// Branch modes
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BranchMode {
    LoopBreak(LoopId),
    LoopBreakIntoMulti(LoopId),
    LoopContinue(LoopId),
    LoopContinueIntoMulti(LoopId),
    MergedBranch,
    MergedBranchIntoMulti,
}

#[derive(Debug, PartialEq)]
pub struct LoopBlock<L: RelooperLabel> {
    pub loop_id: LoopId,
    pub inner: Box<ShapedBlock<L>>,
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
    Multiple(L),
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
    ForwardMultiViaNext(L),
    LoopBreak(LoopId),
    LoopBreakIntoMulti(LoopId),
    LoopContinue(LoopId),
    LoopContinueIntoMulti(LoopId),
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
    counter: LoopId,
    graph: Graph<Node<L>, Edge<L>>,
    graph_root: NodeIndex,
    root: NodeIndex,
}

impl<L: RelooperLabel> Relooper<L> {
    fn new(blocks: BTreeMap<L, Vec<L>>, root_label: L) -> Relooper<L> {
        let mut graph = Graph::new();
        let mut nodes = FnvHashMap::default();

        // Add nodes for each block
        for (&label, branches) in &blocks {
            nodes.insert(label, graph.add_node(if branches.len() > 1 { Node::Multiple(label) } else { Node::Basic(label) }));
        }

        // Add the edges
        for (label, branches) in &blocks {
            for branch in branches {
                graph.add_edge(nodes[&label], nodes[&branch], Edge::Forward);
            }
        }

        // Add a root node to the graph, in order to handle when the first label is a loop
        let graph_root = graph.add_node(Node::Root);
        graph.add_edge(graph_root, nodes[&root_label], Edge::Forward);

        Relooper {
            counter: 0,
            graph,
            graph_root,
            root: nodes[&root_label],
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

                // Go through all the incoming edges and find the loop headers
                let mut loop_headers = FnvHashSet::default();
                let mut loop_parents = FnvHashSet::default();
                for &node in &scc {
                    for edge in self.graph.edges_directed(node, Incoming) {
                        if !scc.contains(&edge.source()) {
                            loop_headers.insert(node);
                            loop_parents.insert(edge.source());
                        }
                    }
                }
                let loop_headers = Vec::from_iter(loop_headers);
                let loop_parents = Vec::from_iter(loop_parents);

                let scc_test = |i| !&scc.contains(&i);
                let (loop_node, loop_id) = self.make_loop(&loop_headers, &loop_parents, &scc, scc_test);

                // Fix edges which are branching outside the loop
                let filtered_graph = EdgeFiltered::from_fn(&self.graph, filter_edges);
                let dominators = algo::dominators::simple_fast(&filtered_graph, self.graph_root);

                // Walk through the graph manually
                let loop_at_root = loop_headers.contains(&self.root);
                let mut stack = vec![loop_node];
                let mut discovered = self.graph.visit_map();
                let mut set_next_to_undominated = false;
                let mut potential_next = None;

                'search_loop: while let Some(node) = stack.pop() {
                    if discovered.visit(node) {
                        // Mark a non-scc node as something to potentially pull out of the loop below
                        // TODO: Can we do better by measuring the 'weight' of a node and its descendants?
                        if node != loop_node && potential_next == None && !&scc.contains(&node) {
                            potential_next = Some(node);
                            // If the root node is a loop, stop once we've found one
                            if loop_at_root {
                                break 'search_loop;
                            }
                        }

                        let mut edges = self.graph.neighbors(node).detach();
                        'edge_loop: while let Some((edge, target)) = edges.next(&self.graph) {
                            if let Edge::Forward = self.graph[edge] {
                                // When the root node is a loop there can't be any un-dominated nodes, so just push to the stack
                                if loop_at_root {
                                    if !discovered.is_visited(&target) {
                                        stack.push(target);
                                    }
                                    continue 'edge_loop;
                                }

                                let target_dominators = dominators.strict_dominators(target).unwrap();
                                for dom in target_dominators {
                                    if dom == loop_node {
                                        // This node is dominated by the structural dominator, so add it to the stack
                                        if !discovered.is_visited(&target) {
                                            stack.push(target);
                                        }
                                        continue 'edge_loop;
                                    }
                                }

                                // Not dominated, so convert the edges
                                set_next_to_undominated = true;
                                self.graph[edge] = Edge::LoopBreak(loop_id);
                                // Add a next edge to the dominator if there isn't one already
                                let dominator = dominators.immediate_dominator(target).unwrap();
                                if !self.graph.contains_edge(dominator, target) {
                                    self.graph.add_edge(dominator, target, Edge::Next);
                                }
                            }
                        }
                    }
                }

                // If we didn't set the next node to an un-dominated node we can potentially pull out one non-scc node instead
                if !set_next_to_undominated {
                    if let Some(node) = potential_next {
                        // Patch incoming edges to the node
                        let mut edges = self.graph.neighbors_directed(node, Incoming).detach();
                        while let Some((edge, _)) = edges.next(&self.graph) {
                            self.graph[edge] = Edge::LoopBreak(loop_id);
                        }
                        // Add an edge to the loop/its parent
                        if loop_at_root {
                            self.graph.update_edge(loop_node, node, Edge::Next);
                        }
                        else {
                            let loop_parent = dominators.immediate_dominator(loop_node).unwrap();
                            self.graph.update_edge(loop_parent, node, Edge::Next);
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
        struct MergingBranches {
            dominator: NodeIndex,
            dominator_order: usize,
            dominated_nodes: Vec<NodeIndex>,
            parent_nodes: FnvHashSet<NodeIndex>,
        }

        // Get the list of nodes in topological order and the dominators list
        let filtered_graph = EdgeFiltered::from_fn(&self.graph, filter_edges);
        let dominators = algo::dominators::simple_fast(&filtered_graph, self.graph_root);
        let sorted_nodes = algo::toposort(&filtered_graph, None).unwrap();
        let mut nodes_to_process = FnvHashMap::default();

        // Now go through the nodes in order, looking for those that have multiple parents
        for &node in sorted_nodes.iter() {
            let mut parent_nodes = FnvHashSet::default();
            for edge in self.graph.edges_directed(node, Incoming) {
                match edge.weight() {
                    Edge::Forward | Edge::ForwardMulti(_) | Edge::LoopBreak(_) | Edge::LoopBreakIntoMulti(_) | Edge::Next => {
                        parent_nodes.insert(edge.source());
                    },
                    _ => {},
                }
            }
            if parent_nodes.len() > 1 {
                let dominator_id = dominators.immediate_dominator(node).unwrap();
                if !nodes_to_process.contains_key(&dominator_id) {
                    // Insert into the map the dominator's topological position and an empty vec for the dominated nodes
                    nodes_to_process.insert(dominator_id, MergingBranches {
                        dominator: dominator_id,
                        dominator_order: sorted_nodes.iter().position(|&n| n == dominator_id).unwrap(),
                        dominated_nodes: Vec::default(),
                        parent_nodes: FnvHashSet::default(),
                    });
                }
                let branch = nodes_to_process.get_mut(&dominator_id).unwrap();
                branch.dominated_nodes.push(node);
                for parent in parent_nodes {
                    branch.parent_nodes.insert(parent);
                }
            }
        }
        // Sort the dominator nodes in topological order
        let mut nodes_to_process = Vec::from_iter(nodes_to_process.values());
        nodes_to_process.sort_by(|a, b| a.dominator_order.cmp(&b.dominator_order));

        // Now go through the dominator nodes, processing each merging node it dominates
        for merged_branch in nodes_to_process {
            let dominator = merged_branch.dominator;
            let dominated_nodes = &merged_branch.dominated_nodes;
            let into_multi = dominated_nodes.len() > 1;

            // Handle pre-existing next nodes from loop breaks
            let mut dominator_edges = self.graph.neighbors(dominator).detach();
            while let Some((edge_id, target)) = dominator_edges.next(&self.graph) {
                if let Edge::Next = self.graph[edge_id] {
                    if !dominated_nodes.contains(&target) {
                        panic!("Loop break to a node that is not one of this dominator's branch merges");
                    }
                    if into_multi {
                        // Convert the LoopBreak nodes into LoopBreakIntoMulti if this is a multi node
                        let mut target_edges = self.graph.neighbors_directed(target, Incoming).detach();
                        while let Some((edge_id, _)) = target_edges.next(&self.graph) {
                            match self.graph[edge_id] {
                                Edge::LoopBreak(loop_id) => { self.graph[edge_id] = Edge::LoopBreakIntoMulti(loop_id); },
                                _ => {},
                            };
                        }
                    }
                }
            }

            // Simple case - only one merged branch, or branches that can't reach each other
            if !into_multi || !self.can_nodes_reach_each_other(&dominated_nodes) {
                for &node in dominated_nodes {
                    // Add the next node
                    self.graph.add_edge(dominator, node, Edge::Next);
                    // Patch the exisiting incoming edges
                    let mut incoming_edges = self.graph.neighbors_directed(node, Incoming).detach();
                    while let Some((edge, _)) = incoming_edges.next(&self.graph) {
                        match self.graph[edge] {
                            Edge::Forward => self.graph[edge] = if into_multi { Edge::MergedBranchIntoMulti } else { Edge::MergedBranch },
                            Edge::LoopBreak(loop_id) => self.graph[edge] = if into_multi { Edge::LoopBreakIntoMulti(loop_id) } else { Edge::LoopBreak(loop_id) },
                            _ => {},
                        };
                    }
                }
                continue;
            }

            // Multiple nodes get turned into a LoopMulti
            let loop_headers = dominated_nodes.iter().map(|&i| i).collect();
            let loop_parents: Vec<NodeIndex> = merged_branch.parent_nodes.difference(&FnvHashSet::from_iter(dominated_nodes.iter().map(|&i| i))).map(|&i| i).collect();

            let loop_parents_test = |i| loop_parents.contains(&i);
            self.make_loop(&loop_headers, &loop_parents, &loop_headers, loop_parents_test);
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
            let mut immediate_entries = FnvHashSet::default();
            let mut next_entries = FnvHashSet::default();
            let mut outgoing_branches = FnvHashMap::default();
            for edge in self.graph.edges(node_id) {
                if let Edge::Removed = edge.weight() {
                    continue;
                }
                let target = edge.target();
                let mut add_branch = |target, branch| outgoing_branches.insert(self.get_basic_node_label(target), branch);
                match edge.weight() {
                    Edge::Forward | Edge::ForwardMulti(_) => { immediate_entries.insert(target); },
                    Edge::Next => { next_entries.insert(target); },
                    Edge::ForwardMultiViaNext(label) => { outgoing_branches.insert(*label, BranchMode::MergedBranchIntoMulti); },
                    Edge::LoopBreak(loop_id) => { add_branch(target, BranchMode::LoopBreak(*loop_id)); },
                    Edge::LoopBreakIntoMulti(loop_id) => { add_branch(target, BranchMode::LoopBreakIntoMulti(*loop_id)); },
                    Edge::LoopContinue(loop_id) => { add_branch(target, BranchMode::LoopContinue(*loop_id)); },
                    Edge::LoopContinueIntoMulti(loop_id) => { add_branch(target, BranchMode::LoopContinueIntoMulti(*loop_id)); },
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
                        immediate: self.output(immediate_entries),
                        next: self.output(next_entries),
                        branches: outgoing_branches,
                    })))
                },
                // This node originally branched, even though there may not be multiple immediate entries now
                Node::Multiple(label) => {
                    Some(Box::new(ShapedBlock::Simple(SimpleBlock {
                        label: *label,
                        // Return a MultipleBlock even if there's now only one immediate entry
                        immediate: if immediate_entries.len() > 0 {
                            Some(Box::new(ShapedBlock::Multiple(MultipleBlock {
                                handled: self.output_multiple_handled(immediate_entries),
                            })))
                        } else { None },
                        next: self.output(next_entries),
                        branches: outgoing_branches,
                    })))
                },
                Node::Loop(loop_id) => {
                    Some(Box::new(ShapedBlock::Loop(LoopBlock {
                        loop_id: *loop_id,
                        inner: self.output(immediate_entries).unwrap(),
                        next: self.output(next_entries),
                    })))
                },
                Node::LoopMulti(loop_id) => {
                    let handled = self.output_multiple_handled(immediate_entries);
                    Some(Box::new(ShapedBlock::Loop(LoopBlock {
                        loop_id: *loop_id,
                        inner: Box::new(ShapedBlock::Multiple(MultipleBlock {
                            handled,
                        })),
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

    // Can any of these nodes reach any other?
    fn can_nodes_reach_each_other(&self, nodes: &Vec<NodeIndex>) -> bool {
        // Filter the graph to ignore processed back edges
        let filtered_graph = EdgeFiltered::from_fn(&self.graph, filter_edges);
        let mut space = algo::DfsSpace::new(&filtered_graph);
        for &x in nodes {
            for &y in nodes {
                if x == y {
                    continue;
                }
                if algo::has_path_connecting(&filtered_graph, x, y, Some(&mut space)) {
                    return true;
                }
            }
        }
        return false;
    }

    fn get_basic_node_label(&self, id: NodeIndex) -> L {
        match self.graph[id] {
            Node::Basic(label) | Node::Multiple(label) => label,
            Node::Loop(_) => {
                let mut edges = self.graph.neighbors(id).detach();
                while let Some((edge, target)) = edges.next(&self.graph) {
                    if let Edge::Forward = self.graph[edge] {
                        match self.graph[target] {
                            Node::Basic(label) | Node::Multiple(label) => return label,
                            other => panic!("Cannot get label of node inside loop: {:?}", other),
                        }
                    }
                }
                panic!("No inner node within loop node: {:?}", self.graph[id]);
            },
            other => panic!("Cannot get label of node: {:?}", other),
        }
    }

    // Return the SCCs over a filtered graph
    fn graph_sccs(&self) -> Vec<Vec<NodeIndex>> {
        // Filter the graph to ignore processed back edges
        let filtered_graph = EdgeFiltered::from_fn(&self.graph, filter_edges);
        algo::kosaraju_scc(&filtered_graph)
    }

    // Make a loop
    // scc can be empty when the loop is for handling merging branches that can reach each other
    fn make_loop<F: Fn(NodeIndex) -> bool>(&mut self, loop_headers: &Vec<NodeIndex>, loop_parents: &Vec<NodeIndex>, loop_nodes: &Vec<NodeIndex>, loop_parent_filter: F) -> (NodeIndex, LoopId) {
        let multi_loop = loop_headers.len() > 1;

        // Add the new node
        let loop_id = self.counter;
        self.counter += 1;
        let loop_node = self.graph.add_node(if multi_loop { Node::LoopMulti(loop_id) } else { Node::Loop(loop_id) });

        // Process the incoming edges
        for &node in loop_headers {
            let mut edges = self.graph.neighbors_directed(node, Incoming).detach();
            while let Some((edge_id, parent)) = edges.next(&self.graph) {
                if loop_parent_filter(parent) {
                    match self.graph[edge_id] {
                        // Forward edges get replaced
                        Edge::Forward => {
                            self.graph.add_edge(parent, loop_node, if multi_loop { Edge::ForwardMulti(self.get_basic_node_label(node)) } else { Edge::Forward });
                            self.graph[edge_id] = Edge::Removed;
                        },
                        Edge::ForwardMulti(_) => unreachable!("A loop node should never be a loop header of a second loop"),
                        // Next edges get removed
                        Edge::Next => {
                            self.graph[edge_id] = Edge::Removed;
                        },
                        // Other edges are left as they are
                        _ => {},
                    };
                }
            }
        }

        // Now add edges from the new node to the loop header(s)
        for &header in loop_headers {
            self.graph.add_edge(loop_node, header, Edge::Forward);
        }

        // If the loop parent of a LoopMulti was a Multiple, check if we can turn it into a Simple now.
        if multi_loop {
            for &node in loop_parents {
                if let Node::Multiple(_) = self.graph[node] {
                    let mut neighbors = FnvHashSet::default();
                    for edge in self.graph.edges(node) {
                        match edge.weight() {
                            // Ignore Next and Removed edges, but include everything else
                            Edge::Next | Edge::Removed => {},
                            _ => { neighbors.insert(edge.target()); },
                        };
                    }
                    if neighbors.len() == 1 {
                        self.graph[node] = Node::Basic(self.get_basic_node_label(node));
                    }
                }
            }
        }

        // If branching to a loop header, convert to a back edge
        for &node in loop_nodes {
            let mut edges = self.graph.neighbors(node).detach();
            while let Some((edge, target)) = edges.next(&self.graph) {
                if loop_headers.contains(&target) {
                    self.graph[edge] = if multi_loop { Edge::LoopContinueIntoMulti(loop_id) } else { Edge::LoopContinue(loop_id) };
                }
            };
        }

        // If we have multiple parents, fix the ForwardMulti edges so that the loop won't be outputted multiple times
        if loop_parents.len() > 1 {
            let filtered_graph = EdgeFiltered::from_fn(&self.graph, filter_edges);
            let dominators = algo::dominators::simple_fast(&filtered_graph, self.graph_root);
            let dominator = dominators.immediate_dominator(loop_node).unwrap();
            for &node in loop_parents {
                let mut edges = self.graph.neighbors(node).detach();
                while let Some((edge, _)) = edges.next(&self.graph) {
                    if let Edge::ForwardMulti(label) = self.graph[edge] {
                        self.graph[edge] = Edge::ForwardMultiViaNext(label);
                    }
                };
            }
            self.graph.add_edge(dominator, loop_node, Edge::Next);
        }

        (loop_node, loop_id)
    }

    fn output_multiple_handled(&self, entries: Vec<NodeIndex>) -> Vec<HandledBlock<L>> {
        let mut handled = Vec::default();
        for entry in entries {
            handled.push(HandledBlock {
                labels: match self.graph[entry] {
                    Node::Basic(label) | Node::Multiple(label) => vec![label],
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