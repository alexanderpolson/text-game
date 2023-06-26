use std::cell::RefCell;
use std::rc::Rc;

pub struct Node<NodeElement, EdgeElement> {
    pub element: NodeElement,
    edges: Vec<Edge<NodeElement, EdgeElement>>,

}

impl<NodeElement, EdgeElement: PartialEq + Clone> Node<NodeElement, EdgeElement> {
    pub fn new(element: NodeElement) -> Self {
        Node {
            element,
            edges: vec![],
        }
    }

    pub fn edge_elements(&self) -> Vec<EdgeElement> {
        self.edges.iter().map(|edge| edge.element.clone()).collect::<Vec<EdgeElement>>()
    }

    pub fn insert_edge(&mut self, edge: EdgeElement, node: &Rc<RefCell<Node<NodeElement, EdgeElement>>>) {
        self.edges.push(Edge::new(edge, node.clone()));
    }

    pub fn node_for_edge_element(&self, element: EdgeElement) -> Option<Rc<RefCell<Node<NodeElement, EdgeElement>>>> {
        self.edges.iter()
            .find(|item| item.matches(&element))
            .map(|item| item.destination_node.clone())
    }
}

/// Represents an edge between nodes in a single direction.
pub struct Edge<NodeElement, EdgeElement> {
    pub element: EdgeElement,
    destination_node: Rc<RefCell<Node<NodeElement, EdgeElement>>>,
}

impl<NodeElement, EdgeElement: PartialEq + Clone> Edge<NodeElement, EdgeElement> {
    pub fn new(element: EdgeElement, destination_node: Rc<RefCell<Node<NodeElement, EdgeElement>>>) -> Self {
        Edge {
            destination_node: destination_node.clone(),
            element,
        }
    }

    pub fn matches(&self, element: &EdgeElement) -> bool {
        self.element == *element
    }
}

pub struct Graph<NodeElement, EdgeElement> {
    root_node: Rc<RefCell<Node<NodeElement, EdgeElement>>>,
    current_node: Rc<RefCell<Node<NodeElement, EdgeElement>>>,
}

impl<NodeElement, EdgeElement: PartialEq + Clone> Graph<NodeElement, EdgeElement> {
    pub fn new(root_node: Node<NodeElement, EdgeElement>) -> Self {
        let root_node_ptr = Rc::new(RefCell::new(root_node));
        Graph {
            root_node: root_node_ptr.clone(),
            current_node: root_node_ptr.clone(),
        }
    }

    pub fn root_node(&self) -> Rc<RefCell<Node<NodeElement, EdgeElement>>> {
        self.root_node.clone()
    }

    pub fn current_node(&self) -> Rc<RefCell<Node<NodeElement, EdgeElement>>> {
        self.current_node.clone()
    }

    pub fn insert_edge(&mut self, edge: EdgeElement, node: &Rc<RefCell<Node<NodeElement, EdgeElement>>>) {
        self.current_node.borrow_mut().insert_edge(edge, node);
    }

    pub fn traverse(&mut self, edge: EdgeElement) -> Option<Rc<RefCell<Node<NodeElement, EdgeElement>>>> {
        // The first block needs to borrow the current node to try and find a match. Once the block
        // goes out of scope, the borrow ends and the second block is responsible for assigning the
        // matched node as the current node. Without the separate blocks, the compiler complains
        // about self.current already being borrowed when trying to assign to it.
        match {
            let current_node = self.current_node.borrow();
            match current_node.node_for_edge_element(edge) {
                Some(matched_node) => {
                    Some(matched_node.clone())
                }
                None => None
            }
        } {
            Some(matched_node) => {
                self.current_node = matched_node.clone();
                Some(matched_node.clone())
            }
            None => None
        }
    }

    pub fn reset(&mut self) {
        self.current_node = self.root_node().clone();
    }
}