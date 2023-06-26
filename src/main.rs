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

impl<NodeElement, EdgeElement: PartialEq> Node<NodeElement, EdgeElement> {
    pub fn new(element: NodeElement) -> Self {
        Node {
            element,
            edges: vec![],
        }
    }

    pub fn insert_edge(mut self, edge: Edge<NodeElement, EdgeElement>) {
        self.edges.push(edge);
    }

    pub fn node_for_edge_element(&self, element: EdgeElement) -> Option<Rc<RefCell<Node<NodeElement, EdgeElement>>>> {
        self.edges.iter()
            .find(|item| item.element == element)
            .map(|item| item.destination_node.clone())
    }
}

/// Represents an edge between nodes in a single direction.
struct Edge<NodeElement, EdgeElement> {
    element: EdgeElement,
    destination_node: Rc<RefCell<Node<NodeElement, EdgeElement>>>,
}

impl<NodeElement, EdgeElement: PartialEq> Edge<NodeElement, EdgeElement> {
    pub fn new(element: EdgeElement, destination_node: Rc<RefCell<Node<NodeElement, EdgeElement>>>) -> Self {
        Edge {
            destination_node: destination_node.clone(),
            element,
        }
    }

    pub fn matches(self, element: EdgeElement) -> bool {
        self.element == element
    }
}

struct Graph<NodeElement, EdgeElement> {
    root_node: Rc<RefCell<Node<NodeElement, EdgeElement>>>,
    current_node: Rc<RefCell<Node<NodeElement, EdgeElement>>>,
}

impl<NodeElement, EdgeElement: PartialEq> Graph<NodeElement, EdgeElement> {

    pub fn new(root_node: Node<NodeElement, EdgeElement>) -> Self {
        let root_node_ptr = Rc::new(RefCell::new(root_node));
        Graph {
            root_node: root_node_ptr.clone(),
            current_node: root_node_ptr.clone(),
        }
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
                },
                None => None
            }
        } {
            Some(matched_node) => {
                self.current_node = matched_node.clone();
                Some(matched_node.clone())
            },
            None => None
        }
    }

    pub fn reset(mut self) {
        self.root_node = self.current_node;
    }
}

fn prompt(prompt: &str) -> String {
    print!("{} ", prompt);
    let _ = stdout().flush().expect("Failure flushing stdout");
    let mut input = String::new();
    let _ = stdin().read_line(&mut input).expect("Failure getting STDIN.");
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

fn print_current_location(graph: &StringGraph) {
    // Borrowing example
    // https://www.reddit.com/r/rust/comments/6q4uqc/help_whats_the_best_way_to_join_an_iterator_of/
    let location = &graph.current_node.borrow();
    let possible_directions = location.edges.iter().map(|edge| edge.element.as_str().clone()).collect::<Vec<&str>>().join(", ");
    println!(r#"
Current Location:
    Description: {}
    Possible Directions: {}"#
             , location.element, possible_directions);
}

fn location_edit_menu(graph: &StringGraph) -> String {
    print_current_location(graph);
    println!(r#"
1. Update description.
2. Connect new location.
3. Move
4. Enter interactive mode
x. Exit"#);
    return prompt_with_options(PROMPT, vec!["1", "2", "3", "4", "x"]);
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
    let new_location =
        Rc::new(RefCell::new(StringNode::new(prompt("Enter the description for the new location:"))));
    let new_direction = prompt("Enter the direction that will take you to the new location:");
    {
        let current_node = &graph.current_node;
        let mut borrowed_node = current_node.borrow_mut();
        let to_edge = StringEdge::new(new_direction.clone(), new_location.clone());
        // TODO: Why doesn't insert_edge work here? It says a move is occurring, but I don't understand.
        // graph.current_node.borrow().insert_edge(StringEdge::new(new_direction, new_location));
        borrowed_node.edges.push(to_edge);

        loop {
            match prompt_with_options("Do you want to be able to get back to the original location (Y/N)?", vec!["y", "Y", "n", "N"]).as_str() {
                "y" | "Y" => {
                    let return_direction = prompt("Enter the return direction that will take you back:");
                    new_location.borrow_mut().edges.push(StringEdge::new(return_direction, current_node.clone()));
                    break;
                }
                "n" | "N" => break,
                _ => (),
            }
        }
    }

    graph.traverse(new_direction.clone());
}

fn move_to_location(graph: &mut StringGraph) -> bool {
    loop {
        let desired_direction = prompt("Which way do you want to go? ");
        if desired_direction == "X".to_string() {
            return false;
        }

        // TODO: If there aren't any valid directions, immediately exit.
        // TODO: If there's only one direction, just use it.
        match graph.traverse(desired_direction) {
            Some(_) => return true,
            None => println!("That's not a valid direction. Give it another go...")
        }
    }
}

fn interactive_mode(graph: &mut StringGraph) {
    loop {
        print_current_location(graph);
        // TODO: Keep asking for a direction to go until a valid one is chosen.
        // TODO: Encapsulate this into a function.
        if !move_to_location(graph) {
            break;
        }
    }
}

fn main() {
    let mut graph = StringGraph::new(StringNode::new(prompt("Enter the description of your first node:")));

    // TODO: Add default data.

    loop {
        match location_edit_menu(&graph).as_str() {
            "1" => update_location_description(&mut graph),
            "2" => connect_new_location(&mut graph),
            // TODO: Connect existing location
            // Will need to navivate to it somehow, unless referring by some id?
            "3" => {
                move_to_location(&mut graph);
                ()
            },
            "4" => interactive_mode(&mut graph),
            "X" | "x" => break,
            _ => ()
        }
    }
}
