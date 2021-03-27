/*

Custom graph searches
=====================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use petgraph::prelude::*;
use petgraph::EdgeType;
use petgraph::graph::IndexType;
use petgraph::visit::{GraphRef, Visitable, VisitMap};

// Dfs with edge filtering and adding to the stack *after* the node is returned
#[derive(Clone, Debug)]
pub struct FilteredDfs<E, Ix, VM> {
    // The stack of nodes to visit
    pub stack: Vec<NodeIndex<Ix>>,
    // The map of discovered nodes
    pub discovered: VM,
    // The filter function
    filter_fn: fn(E) -> bool,
    // The last node
    last_node: Option<NodeIndex<Ix>>,
}

impl<E, Ix, VM> FilteredDfs<E, Ix, VM>
where 
    E: Copy + PartialEq,
    Ix: IndexType,
    VM: VisitMap<NodeIndex<Ix>>,
{
    // Create a new **Dfs** using the graph's visitor map, and no stack.
    pub fn empty<G>(graph: G, filter_fn: fn(E) -> bool) -> Self
    where G: GraphRef + Visitable<NodeId = NodeIndex<Ix>, Map = VM>,
    {
        FilteredDfs {
            stack: Vec::new(),
            discovered: graph.visit_map(),
            filter_fn,
            last_node: None,
        }
    }

    // Return the next node in the dfs, or **None** if the traversal is done.
    pub fn next<N, Ty>(&mut self, graph: &Graph<N, E, Ty, Ix>) -> Option<NodeIndex<Ix>>
    where Ty: EdgeType
    {
        // If we have a previous node, add its filtered neighbours to the stack
        if let Some(last_node) = self.last_node {
            let mut edges = graph.neighbors(last_node).detach();
            while let Some((edge, target)) = edges.next(&graph) {
                if !(self.filter_fn)(graph[edge]) {
                    continue;
                }
                if !self.discovered.is_visited(&target) {
                    self.stack.push(target);
                }
            }
            self.last_node = None;
        }
        while let Some(node) = self.stack.pop() {
            if self.discovered.visit(node) {
                self.last_node = Some(node);
                return Some(node);
            }
        }
        None
    }
}