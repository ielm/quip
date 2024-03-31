use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq)]
pub struct GraphNode {
    pub val: i32,
    pub neighbors: Vec<Rc<RefCell<GraphNode>>>,
}

impl GraphNode {
    #[inline]
    pub fn new(val: i32) -> Self {
        GraphNode {
            val,
            neighbors: Vec::new(),
        }
    }
}

pub fn to_graph(vec: Vec<(i32, Vec<i32>)>) -> Vec<Rc<RefCell<GraphNode>>> {
    // Create a vector of `GraphNode` references wrapped in `Rc<RefCell<_>>`
    // for each value in the input vector.
    let nodes = vec
        .iter()
        .map(|(val, _)| Rc::new(RefCell::new(GraphNode::new(*val))))
        .collect::<Vec<_>>();

    // Create a map to associate each node's value with its index in `nodes`.
    let mut index_map = std::collections::HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        index_map.insert(node.borrow().val, i);
    }

    // Iterate over the input vector to establish neighbors for each node.
    for (val, neighbors) in &vec {
        // Find the index of the current node using its value.
        if let Some(&index) = index_map.get(val) {
            // Retrieve a reference to the current `GraphNode`.
            let node = &nodes[index];

            // Iterate over the neighbor values.
            for neighbor_val in neighbors {
                // Find the index of the neighbor node.
                if let Some(&neighbor_index) = index_map.get(neighbor_val) {
                    // Add the neighbor to the current node's neighbor list.
                    node.borrow_mut()
                        .neighbors
                        .push(nodes[neighbor_index].clone());
                }
            }
        }
    }

    // Return the vector of `GraphNode` references.
    nodes
}

#[macro_export]
macro_rules! graph {
    ($($val:expr => [$($neighbors:expr),*]),*) => {
        {
            let vec = vec![$(($val, vec![$($neighbors),*])),*];
            to_graph(vec)
        }
    };
}
