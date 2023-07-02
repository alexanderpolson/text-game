use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;
use std::rc::Rc;

use linked_hash_map::LinkedHashMap;
use serde::{Serialize, Serializer};
use serde::ser::SerializeSeq;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct Node<NodeElement: Serialize, EdgeElement: Hash + Eq + PartialEq + Clone> {
    pub id: String,
    pub element: NodeElement,
    edges: HashMap<EdgeElement, String>,
}

impl<NodeElement: Serialize, EdgeElement: Eq + Hash + Clone> Node<NodeElement, EdgeElement> {
    pub fn new(element: NodeElement) -> Self {
        Node {
            id: Uuid::new_v4().to_string(),
            element,
            edges: HashMap::new(),
        }
    }

    pub fn edge_elements(&self) -> Vec<EdgeElement> {
        self.edges.iter().map(|(element, _)| element.clone()).collect::<Vec<EdgeElement>>()
    }

    pub fn insert_edge(&mut self, element: EdgeElement, node_id: String) {
        self.edges.insert(element.clone(), node_id);
    }

    pub fn node_for_edge_element(&self, element: &EdgeElement) -> Option<String> {
        match self.edges.get(&element) {
            Some(edge_node_id) => Some(edge_node_id.clone()),
            None => None
        }
    }
}

type NodeRef<NodeElement, EdgeElement> = Rc<RefCell<Node<NodeElement, EdgeElement>>>;


// TODO: Implement Serialize and Deserialize
// https://stackoverflow.com/a/51284093/1060627

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

    pub fn current_node(&self) -> Rc<RefCell<Node<NodeElement, EdgeElement>>> {
        self.nodes[&self.current_node_id].clone()
    }

    pub fn insert_edge_to_new_node(&mut self, edge: EdgeElement, node: NodeElement) -> String {
        let new_node = Node::new(node);
        let new_node_id = new_node.id.clone();
        self.current_node().borrow_mut().insert_edge(edge, new_node_id.clone());
        self.nodes.insert(new_node_id.clone(), Rc::new(RefCell::new(new_node)));
        new_node_id
    }

    pub fn insert_edge_to_existing_node(&mut self, edge: EdgeElement, node_id: String) {
        self.current_node().borrow_mut().insert_edge(edge, node_id.clone());
    }

    pub fn nodes(&self) -> impl Iterator<Item=NodeRef<NodeElement, EdgeElement>> + '_ {
        self.nodes.iter()
            .map(|(_id, node_ref)| node_ref.clone())
    }

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

    pub fn reset(&mut self) {
        self.current_node_id = self.root_node_id.clone();
    }
}

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