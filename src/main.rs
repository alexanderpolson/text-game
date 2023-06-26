/*
 *
 */


use std::cell::RefCell;
use std::io::{stdin, stdout, Write};
use std::rc::Rc;

use crate::graph::{Graph, Node};

mod graph;

const PROMPT: &str = ">";

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
type StringGraph = Graph<String, String>;

fn print_current_location(graph: &StringGraph) {
    // Borrowing example
    // https://www.reddit.com/r/rust/comments/6q4uqc/help_whats_the_best_way_to_join_an_iterator_of/
    let current_node = &graph.current_node();
    let location = current_node.borrow();
    let possible_directions = location.edge_elements().join(", ");
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
5. Reset to root node.
x. Exit"#);
    return prompt_with_options(PROMPT, vec!["1", "2", "3", "4", "5", "x"]);
}

fn update_location_description(graph: &mut StringGraph) {
    println!("Current description:\n{}\n", &graph.current_node().borrow().element);
    {
        let current_node = &graph.current_node();
        let mut borrowed_node = current_node.borrow_mut();
        borrowed_node.element = prompt("Enter the new description of your first node:");
    }
}

fn connect_new_location(graph: &mut StringGraph) {
    let new_location =
        Rc::new(RefCell::new(StringNode::new(prompt("Enter the description for the new location:"))));
    let new_direction = prompt("Enter the direction that will take you to the new location:");
    graph.insert_edge(new_direction.clone(), &new_location);
    // Capture the current_node before traversal just in case the user wants a reverse edge created
    // as well.
    let current_node = graph.current_node();
    graph.traverse(new_direction);
    loop {
        match prompt_with_options("Do you want to be able to get back to the original location (Y/N)?", vec!["y", "Y", "n", "N"]).as_str() {
            "y" | "Y" => {
                let return_direction = prompt("Enter the return direction that will take you back:");
                graph.current_node().borrow_mut().insert_edge(return_direction, &current_node);
                break;
            }
            "n" | "N" => break,
            _ => (),
        }
    }
}

fn move_to_location(graph: &mut StringGraph) -> bool {
    loop {
        let desired_direction = prompt("Which way do you want to go? ");
x        if desired_direction == "X".to_string() {
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
            }
            "4" => interactive_mode(&mut graph),
            "5" => graph.reset(),
            "X" | "x" => break,
            _ => ()
        }
    }
}
