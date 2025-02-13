use crate::{shape::Shape, Buffer, Device, Graph, NodeIdx};

use super::node::Node;

/// Trait for adding a node to a graph.
pub trait AddGraph {
    /// Returns the indices of the node's parents.
    #[inline]
    fn idxs(&self) -> (usize, usize) {
        (0, 0)
    }

    /// Adds a node to the graph.
    #[inline]
    fn add<IdxFrom: NodeIdx>(&self, graph: &mut Graph<IdxFrom>, len: usize) -> Node {
        let (lhs_idx, rhs_idx) = self.idxs();
        graph.add_node(len, lhs_idx, rhs_idx)
    }
}

impl AddGraph for () {
    #[inline]
    fn add<IdxFrom: NodeIdx>(&self, graph: &mut Graph<IdxFrom>, len: usize) -> Node {
        graph.add_leaf(len)
    }
}

// Unary operation
impl AddGraph for usize {
    #[inline]
    fn idxs(&self) -> (usize, usize) {
        (*self, *self)
    }
}

impl AddGraph for (usize, usize) {
    #[inline]
    fn idxs(&self) -> (usize, usize) {
        *self
    }
}

impl<'a, T, D: Device, S: Shape> AddGraph for Buffer<'a, T, D, S> {
    #[inline]
    fn idxs(&self) -> (usize, usize) {
        (self.id().idx, self.id().idx)
    }
}

impl<'a, T, D: Device, S: Shape> AddGraph for &Buffer<'a, T, D, S> {
    #[inline]
    fn idxs(&self) -> (usize, usize) {
        (self.id().idx, self.id().idx)
    }
}

impl<'a, T, D: Device, LS: Shape, RS: Shape> AddGraph
    for (&Buffer<'a, T, D, LS>, &Buffer<'a, T, D, RS>)
{
    #[inline]
    fn idxs(&self) -> (usize, usize) {
        (self.0.id().idx, self.1.id().idx)
    }
}
