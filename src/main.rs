/*
 *
 */


use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{stdin, stdout, Write};

use crate::graph::Graph;

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
2. Connect location.
3. Move
4. Enter interactive mode
5. Reset to root node.
6. Save
x. Exit"#);
    return prompt_with_options(PROMPT, vec!["1", "2", "3", "4", "5", "6", "x"]);
}

fn update_location_description(graph: &mut StringGraph) {
    println!("Current description:\n{}\n", &graph.current_node().borrow().element);
    {
        let current_node = &graph.current_node();
        let mut borrowed_node = current_node.borrow_mut();
        borrowed_node.element = prompt("Enter the new description of your first node:");
    }
}

fn connect_location(graph: &mut StringGraph) {
    let new_direction = prompt("Enter the direction that will take you to the new location:");

    // TODO: New or existing location?
    loop {
        match prompt_with_options("Create a new location or use an existing location?", vec!["N", "n", "E", "e"]).as_str() {
            "n" | "N" => {
                graph.insert_edge_to_new_node(new_direction.clone(), prompt("Enter the description for the new location:"));
                break;
            }
            "e" | "E" => {
                graph.insert_edge_to_existing_node(new_direction.clone(), select_node(graph));
                break;
            }
            _ => (),
        }
    }
    let original_node = graph.current_node();

    graph.traverse(new_direction);
    loop {
        match prompt_with_options("Do you want to be able to get back to the original location (Y/N)?", vec!["y", "Y", "n", "N"]).as_str() {
            "y" | "Y" => {
                let return_direction = prompt("Enter the return direction that will take you back:");
                let original_node_borrowed = original_node.borrow();
                graph.current_node().borrow_mut().insert_edge(return_direction, original_node_borrowed.id.clone());
                break;
            }
            "n" | "N" => break,
            _ => (),
        }
    }
}

fn select_node(graph: &StringGraph) -> String {
    let mut idx: u32 = 0;
    let mut node_id_index: HashMap<u32, String> = HashMap::new();
    for node in graph.nodes() {
        idx += 1;
        let borrowed_node = node.borrow();
        node_id_index.insert(idx, borrowed_node.id.clone());
        println!("{}. {}", idx, borrowed_node.element);
    }

    loop {
        // TODO: Handle cancel option.
        match prompt("Enter the location number:").parse::<u32>() {
            Ok(selected_idx) => {
                match node_id_index.get(&selected_idx) {
                    Some(node_id) => return node_id.clone(),
                    None => (),
                }
            }
            Err(_) => ()
        }
        println!("Invalid input. Give it another go...")
    }
}

fn move_to_location(graph: &mut StringGraph) -> bool {
    loop {
        let desired_direction = prompt("Which way do you want to go? ");
        if desired_direction.to_uppercase() == "X".to_string() {
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
        if !move_to_location(graph) {
            break;
        }
    }
}

fn save(graph: &StringGraph) {
    match serde_json::to_string(&graph) {
        Ok(data) => {
            println!("{}", data);
            match fs::write("game.json", data.as_bytes()) {
                Ok(_) => (),
                Err(e) => println!("An error occurred: {:#?}", e)
            }
        }
        Err(e) => println!("An error occurred while writing: {:#?}", e),
    }
}

fn main() {
    let mut graph =
        StringGraph::new(prompt("Enter the description of your first node:"));

    loop {
        match location_edit_menu(&graph).as_str() {
            "1" => update_location_description(&mut graph),
            "2" => connect_location(&mut graph),
            "3" => {
                move_to_location(&mut graph);
                ()
            }
            "4" => interactive_mode(&mut graph),
            "5" => graph.reset(),
            "6" => save(&graph),
            "X" | "x" => break,
            _ => ()
        }
    }
}
