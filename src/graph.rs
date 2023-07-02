use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a node in a Graph along with "pointers" to all of its edges based on their ids.
#[derive(Debug, Deserialize, Serialize)]
pub struct Node<NodeElement: Serialize, EdgeElement: Hash + Eq + PartialEq + Clone> {
    /// Uniquely identifies this node.
    pub id: String,
    /// The data associated with this node.
    pub element: NodeElement,
    /// Maps an edge to the id of the node that the edge points to.
    edges: HashMap<EdgeElement, String>,
}

impl<NodeElement: Serialize, EdgeElement: Eq + Hash + Clone> Node<NodeElement, EdgeElement> {
    /// Create a new Node based on the element type being stored in the node.
    pub fn new(element: NodeElement) -> Self {
        Node {
            id: Uuid::new_v4().to_string(),
            element,
            edges: HashMap::new(),
        }
    }

    // Returns an Iterator over all of the edge elements for this Node.
    pub fn edge_elements(&self) -> impl Iterator<Item=EdgeElement> + '_ {
        self.edges.iter().map(|(element, _)| element.clone())
    }

    /// Insert a new edge that leads to the Node for the provided id.
    pub fn insert_edge(&mut self, element: EdgeElement, node_id: String) {
        self.edges.insert(element.clone(), node_id);
    }

    /// Find the id for the Node that is connected to this Node via the provided edge, if one
    /// exists.
    pub fn node_for_edge_element(&self, element: &EdgeElement) -> Option<String> {
        match self.edges.get(&element) {
            Some(edge_node_id) => Some(edge_node_id.clone()),
            None => None
        }
    }
}

/// Shortcut for the pointers that are used for Nodes throughout the implementation.
type NodeRef<NodeElement, EdgeElement> = Rc<RefCell<Node<NodeElement, EdgeElement>>>;

/// A Graph structure that allows traversal from one node to another. As the Graph is traversed, any
/// changes that are made, are typically made to the current node.
pub struct Graph<NodeElement: Serialize, EdgeElement: Hash + Eq + PartialEq + Clone> {
    root_node_id: String,
    current_node_id: String,
    nodes: LinkedHashMap<String, NodeRef<NodeElement, EdgeElement>>,
}

impl<NodeElement: Serialize, EdgeElement: Eq + Hash + Clone> Graph<NodeElement, EdgeElement> {
    pub fn new(root_node_element: NodeElement) -> Self {
        let root_node = Node::new(root_node_element);
        let mut nodes = LinkedHashMap::new();
        let root_node_id = root_node.id.clone();
        nodes.insert(root_node_id.clone(), Rc::new(RefCell::new(root_node)));
        Graph {
            root_node_id: root_node_id.clone(),
            current_node_id: root_node_id,
            nodes,
        }
    }

    /// Get a reference to the node for the current position in the Graph.
    pub fn current_node(&self) -> Rc<RefCell<Node<NodeElement, EdgeElement>>> {
        self.nodes[&self.current_node_id].clone()
    }

    /// Insert an edge from the current Node that leads to a brand new Node with the provided
    /// element.
    pub fn insert_edge_to_new_node(&mut self, edge: EdgeElement, node: NodeElement) -> String {
        let new_node = Node::new(node);
        let new_node_id = new_node.id.clone();
        self.current_node().borrow_mut().insert_edge(edge, new_node_id.clone());
        self.nodes.insert(new_node_id.clone(), Rc::new(RefCell::new(new_node)));
        new_node_id
    }

    /// Insert an edge from the current Node to an existing node for the provided `node_id`.
    pub fn insert_edge_to_existing_node(&mut self, edge: EdgeElement, node_id: String) {
        self.current_node().borrow_mut().insert_edge(edge, node_id.clone());
    }

    /// Returns an Iterator over all the Nodes in the Graph in insertion order.
    pub fn nodes(&self) -> impl Iterator<Item=NodeRef<NodeElement, EdgeElement>> + '_ {
        self.nodes.iter()
            .map(|(_id, node_ref)| node_ref.clone())
    }

    /// Update the current node in the Graph to another Node based on the provided edge element that
    /// leads to that Node.
    pub fn traverse(&mut self, edge_element: EdgeElement) -> Option<NodeRef<NodeElement, EdgeElement>> {
        // The first block needs to borrow the current node to try and find a match. Once the block
        // goes out of scope, the borrow ends and the second block is responsible for assigning the
        // matched node as the current node. Without the separate blocks, the compiler complains
        // about self.current already being borrowed when trying to assign to it.
        match {
            let current_node = self.current_node();
            let borrowed_current_node = current_node.borrow();
            borrowed_current_node.node_for_edge_element(&edge_element)
        } {
            Some(matched_node) => {
                self.current_node_id = matched_node.clone();
                Some(self.current_node())
            }
            None => None
        }
    }

    /// Update the position in the Graph to be the root Node.
    pub fn reset(&mut self) {
        self.current_node_id = self.root_node_id.clone();
    }
}

/// JSON serialization for Graph. Derived from
/// [an example from StackOverflow](https://stackoverflow.com/a/51284093/1060627).
impl<NodeElement: Eq + Hash + Clone + Serialize, EdgeElement: Eq + Hash + Clone + Serialize> Serialize for Graph<NodeElement, EdgeElement> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        // Serialize the nodes with their ids.
        let mut node_seq = serializer.serialize_seq(Some(self.nodes.len()))?;
        for (_, node) in &self.nodes {
            let borrowed_node = node.borrow();
            node_seq.serialize_element(borrowed_node.deref())?;
        }
        node_seq.end()
    }
}

struct GraphVisitor<NodeElement: Eq + Hash + Clone + Serialize, EdgeElement: Eq + Hash + Clone + Serialize> {
    marker: PhantomData<fn() -> Graph<NodeElement, EdgeElement>>,
}

impl<NodeElement: Eq + Hash + Clone + Serialize, EdgeElement: Eq + Hash + Clone + Serialize> GraphVisitor<NodeElement, EdgeElement> {
    fn new() -> Self {
        GraphVisitor {
            marker: PhantomData
        }
    }
}

impl<'de, NodeElement: Eq + Hash + Clone + Serialize + Deserialize<'de>, EdgeElement: Eq + Hash + Clone + Serialize + Deserialize<'de>> Visitor<'de> for GraphVisitor<NodeElement, EdgeElement> {
    type Value = Vec<Node<NodeElement, EdgeElement>>;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("A Graph structure.")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
        let mut nodes: Vec<Node<NodeElement, EdgeElement>> = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(node) = seq.next_element::<Node<NodeElement, EdgeElement>>()? {
            nodes.push(node);
        }
        Ok(nodes)
    }
}

impl<'de, NodeElement: Eq + Hash + Clone + Serialize + Deserialize<'de>, EdgeElement: Eq + Hash + Clone + Serialize + Deserialize<'de>> Deserialize<'de> for Graph<NodeElement, EdgeElement> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        match deserializer.deserialize_seq(GraphVisitor::new()) {
            Ok(nodes) => {
                // TODO: Return an error if there aren't any nodes
                let root_node_id = {
                    let root_node = &nodes[0];
                    root_node.id.clone()
                };
                let mut nodes_map: LinkedHashMap<String, NodeRef<NodeElement, EdgeElement>> = LinkedHashMap::with_capacity(nodes.len());
                for node in nodes {
                    nodes_map.insert(node.id.clone(), Rc::new(RefCell::new(node)));
                }
                Ok(Graph {
                    current_node_id: root_node_id.clone(),
                    root_node_id: root_node_id.clone(),
                    nodes: nodes_map,
                })
            }
            Err(e) => Err(e),
        }
    }
}