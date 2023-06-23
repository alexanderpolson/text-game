/*
 *
 */


use std::cell::RefCell;
use std::io::{stdin, stdout, Write};
use std::rc::Rc;

const PROMPT: &str = ">";

struct Node<NodeElement, EdgeElement> {
    element: NodeElement,
    edges: Vec<Edge<NodeElement, EdgeElement>>
}

impl<NodeElement, EdgeElement> Node<NodeElement, EdgeElement> {
    pub fn new(element: NodeElement) -> Self {
        Node {
            element,
            edges: vec![],
        }
    }
    pub fn insert_edge(mut self, edge: Edge<NodeElement, EdgeElement>) {
        self.edges.push(edge);
    }
}

/// Represents an edge between nodes in a single direction.
struct Edge<NodeElement, EdgeElement> {
    element: EdgeElement,
    destination_node: Rc<RefCell<Node<NodeElement, EdgeElement>>>,
}

impl<NodeElement, EdgeElement> Edge<NodeElement, EdgeElement> {
    pub fn new(element: EdgeElement, destination_node: Rc<RefCell<Node<NodeElement, EdgeElement>>>) -> Self {
        Edge {
            destination_node: destination_node.clone(),
            element,
        }
    }
}

struct Graph<NodeElement, EdgeElement> {
    root_node: Rc<RefCell<Node<NodeElement, EdgeElement>>>,
    current_node: Rc<RefCell<Node<NodeElement, EdgeElement>>>,
}

impl<NodeElement, EdgeElement> Graph<NodeElement, EdgeElement> {

    pub fn new(root_node: Node<NodeElement, EdgeElement>) -> Self {
        let root_node_ptr = Rc::new(RefCell::new(root_node));
        Graph {
            root_node: root_node_ptr.clone(),
            current_node: root_node_ptr.clone(),
        }
    }

    pub fn reset(mut self) {
        self.root_node = self.current_node;
    }
}

fn prompt(prompt: &str) -> String {
    print!("{} ", prompt);
    stdout().flush();
    let mut input = String::new();
    stdin().read_line(&mut input);
    return input.trim().to_string();
}

fn prompt_with_options(prompt_text: &str, options: Vec<&str>) -> String {
    loop {
        let input = prompt(prompt_text);
        if options.contains(&input.as_str()) {
            return input;
        }
        println!("Invalid input. Give it another go...");
    }
}

type StringNode = Node<String, String>;
type StringEdge = Edge<String, String>;
type StringGraph = Graph<String, String>;

fn location_edit_menu(graph: &StringGraph) -> String {
    // Borrowing example
    // https://www.reddit.com/r/rust/comments/6q4uqc/help_whats_the_best_way_to_join_an_iterator_of/
    let location = &graph.current_node.borrow();
    let possible_directions = location.edges.iter().map(|edge| edge.element.as_str().clone()).collect::<Vec<&str>>().join(", ");
    println!(r#"
Current Location:
    Description: {}
    Possible Directions: {}"#
             , location.element, ""); //possible_directions);
    println!(r#"
1. Update description.
2. Connect new location.
3. Enter interactive mode
x. Exit"#);
    return prompt_with_options(PROMPT, vec!["1", "2", "3", "x"]);
}

fn update_location_description(graph: &mut StringGraph) {
    let current_node = &graph.current_node;
    println!("Current description:\n{}\n", current_node.borrow().element);
    {
        let mut borrowed_node = current_node.borrow_mut();
        borrowed_node.element = prompt("Enter the new description of your first node:");
    }
}

fn connect_new_location(graph: &mut StringGraph) {
    let mut new_location =
        Rc::new(RefCell::new(StringNode::new(prompt("Enter the description for teh new location:"))));
    let new_direction = prompt("Enter the direction that will take you to the new location:");
    {
        let mut current_node = &graph.current_node;
        let mut borrowed_node = current_node.borrow_mut();
        let to_edge = StringEdge::new(new_direction, new_location.clone());
        // TODO: Why doesn't insert_edge work here? It says a move is occurring, but I don't understand.
        // graph.current_node.borrow().insert_edge(StringEdge::new(new_direction, new_location));
        borrowed_node.edges.push(to_edge);

        loop {
            match (prompt_with_options("Do you want to be able to get back to the original location (Y/N)?", vec!["y", "Y", "n", "N"]).as_str()) {
                "y" | "Y" => {
                    let mut return_direction = prompt("Enter the return direction that will take you back:");
                    new_location.borrow_mut().edges.push(StringEdge::new(return_direction, new_location.clone()));
                    break;
                }
                "n" | "Y" => break,
                _ => (),
            }
        }
    }

    graph.current_node = new_location.clone();
}

fn interactive_mode(graph: &mut StringGraph) {

}

fn main() {
    let mut graph = StringGraph::new(StringNode::new(prompt("Enter the description of your first node:")));

    loop {
        match location_edit_menu(&graph).as_str() {
            "1" => update_location_description(&mut graph),
            "2" => connect_new_location(&mut graph),
            "3" => (),
            "X" | "x" => break,
            _ => ()
        }
    }
}
