use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

// Function to calculate cosine similarity between two slices of f32 values
fn cosine_similarity(vec_a: &[f32], vec_b: &[f32]) -> f32 {
    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for (a, b) in vec_a.iter().zip(vec_b.iter()) {
        dot_product += a * b;
        norm_a += a * a;
        norm_b += b * b;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a.sqrt() * norm_b.sqrt())
}

// Define a Priority enum with three levels: High, Medium, and Low, and derive Serialize and Deserialize for JSON handling, as well as traits for comparison and ordering
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Priority {
    High,
    Medium,
    Low,
}

// Define a Task struct that contains a boolean for completion status, a Priority enum for task priority, and a String for the task description, and derive Serialize and Deserialize for JSON handling, as well as traits for comparison and ordering
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct Task {
    completed: bool,     // false (pending) comes before true (completed) when sorting
    priority: Priority,  // High comes before Medium, which comes before Low when sorting
    description: String, // ALphabetical order of descriptions is used as a tiebreaker when sorting tasks with the same completion status and priority
}

// Implement a constructor method for Task to create a new task with a given description and priority, and set completed to false by default
impl Task {
    fn new(description: String, priority: Priority) -> Task {
        Task {
            completed: false,
            priority,
            description,
        }
    }
}

// Define a TodoList struct that contains a vector of Task structs, and derive Serialize and Deserialize for JSON handling
#[derive(Serialize, Deserialize)]
struct TodoList {
    tasks: Vec<Task>,
}

// Implement methods for TodoList
impl TodoList {
    fn new() -> TodoList {
        // Check if "tasks.json" exists and read its text
        if let Ok(json_data) = fs::read_to_string("tasks.json") {
            // Try to convert the JSON text back into a TodoList struct
            if let Ok(parsed_list) = serde_json::from_str(&json_data) {
                return parsed_list; // Success - return the saved list
            }
        }
        // If the file doesn't exist yet or if parsing fails, return an empty TodoList
        TodoList { tasks: Vec::new() }
    }

    // Helper method to save the file
    fn save_to_file(&self) {
        // Convert the TodoList struct into a JSON string
        let json = serde_json::to_string_pretty(&self).expect("Failed to format JSON.");
        // Write the JSON string to "tasks.json" (creates the file if it doesn't exist)
        fs::write("tasks.json", json).expect("Failed to write to file.");
    }

    // Add a new task to the list and save the updated list to a file
    fn add_task(&mut self, input: String) {
        let input = input.trim();

        // Check for priority suffixes and extract tuples of (description, priority) based on the suffix
        let (description, priority) = if input.ends_with("!high") {
            (
                input.trim_end_matches("!high").trim().to_string(),
                Priority::High,
            )
        } else if input.ends_with("!low") {
            (
                input.trim_end_matches("!low").trim().to_string(),
                Priority::Low,
            )
        } else {
            (
                input.trim_end_matches("!medium").trim().to_string(),
                Priority::Medium,
            )
        };

        self.tasks.push(Task::new(description, priority));
        self.tasks.sort(); // Sort the tasks after adding a new one
        self.save_to_file(); // Save the updated list to a file
        println!("\nTask added successfully.");
    }

    // Delete a task by its index (1-based) and save the updated list to a file
    fn delete_task(&mut self, index: usize) {
        if index == 0 || index > self.tasks.len() {
            println!("\nInvalid task number.");
            return;
        }
        self.tasks.remove(index - 1);
        self.save_to_file();
        println!("Task deleted successfully.");
    }

    // List all tasks with their status (completed or not)
    fn list_tasks(&self) {
        if self.tasks.is_empty() {
            println!("\nThere are no tasks in the to-do list.");
            return;
        }

        // Iterate over the tasks and print their 1-based index, status, description, and priority with colors
        for (index, task) in self.tasks.iter().enumerate() {
            let status = if task.completed { "|X|" } else { "| |" };

            // Format the priority text with colors
            let priority_text = match task.priority {
                Priority::High => "(High)".red().bold(),
                Priority::Medium => "(Medium)".yellow(),
                Priority::Low => "(Low)".green(),
            };

            // If the task is completed, print the line in a dimmed color; otherwise, print it normally
            if task.completed {
                println!(
                    "{}",
                    format!(
                        "{} {} {} {}",
                        index + 1,
                        status,
                        task.description,
                        priority_text
                    )
                    .truecolor(100, 100, 100)
                );
            } else {
                println!(
                    "{} {} {} {}",
                    index + 1,
                    status,
                    task.description,
                    priority_text
                );
            }
        }
    }

    // Mark a task as completed by its index (1-based) and save the updated list to a file
    fn complete_task(&mut self, index: usize) {
        if index == 0 || index > self.tasks.len() {
            println!("\nInvalid task number.");
            return;
        }
        self.tasks[index - 1].completed = true;
        self.tasks.sort(); // Sort the tasks after marking one as completed
        self.save_to_file(); // Save the updated list to a file
        println!("\nTask {} marked as completed.", index);
    }
}

// Simple ASCII art animation of a witch greeting the user
fn witch_greeting_animation() {
    let frames = [
        // Frame 1
        format!(
            "\n               \n               \n        {}      \n       . .     \n               ",
            "*".yellow().bold()
        ),
        // Frame 2
        format!(
            "\n               \n       {}     \n      {} . {}    \n       {}     \n               ",
            ".*.".yellow(),
            "*".yellow(),
            "*".yellow(),
            ".*.".yellow()
        ),
        // Frame 3
        format!(
            "\n               \n      {}    \n      {} {} {}    \n      {}    \n               ",
            "\\ | /".yellow().bold(),
            "-".yellow().bold(),
            "*".white().bold(),
            "-".yellow().bold(),
            "/ | \\".yellow().bold()
        ),
        // Frame 4
        format!(
            "\n         {}    \n        {}    \n      {} \n      {}{}{} {} \n       {}  ",
            "_/".purple(),
            "/ /".purple(),
            "_/____\\_".purple(),
            "~".truecolor(139, 69, 19),
            "(-.-)".green(),
            "~".truecolor(139, 69, 19),
            "/".truecolor(139, 69, 19), // Hair, Face, Hair, Wand
            "/(_)\\/".truecolor(100, 100, 100)
        ),
        // Frame 5
        format!(
            "\n         {}    \n        {}    \n      {} \n      {}{}{} {}{}\n       {}  ",
            "_/".purple(),
            "/ /".purple(),
            "_/____\\_".purple(),
            "~".truecolor(139, 69, 19),
            "(o.o)".green().bold(),
            "~".truecolor(139, 69, 19),
            "/".truecolor(139, 69, 19),
            "*".yellow().bold(),
            "/(_)\\/".truecolor(100, 100, 100)
        ),
    ];

    for frame in frames {
        print!("\x1B[2J\x1B[1;1H");
        println!("Summoning your tasks...\n{}", frame);
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(400));
    }
    thread::sleep(Duration::from_millis(600));
}

// Simple ASCII art animation of a witch disappearing when the user exits the program
fn witch_ending_animation() {
    let frames = [
        // Frame 1
        format!(
            "\n         {}    \n        {}    \n      {} \n      {}{}{} {}{}\n       {}  ",
            "_/".purple(),
            "/ /".purple(),
            "_/____\\_".purple(),
            "~".truecolor(139, 69, 19),
            "(o.o)".green().bold(),
            "~".truecolor(139, 69, 19),
            "/".truecolor(139, 69, 19),
            "*".yellow().bold(),
            "/(_)\\/".truecolor(100, 100, 100)
        ),
        // Frame 2
        format!(
            "\n         {}    \n        {}    \n      {} \n      {}{}{} {}{}\n       {}  ",
            "_/".purple(),
            "/ /".purple(),
            "_/____\\_".purple(),
            "~".truecolor(139, 69, 19),
            "(>.<)".green(),
            "~".truecolor(139, 69, 19),
            "/".truecolor(139, 69, 19),
            "**".yellow().bold(),
            "/(_)\\/".truecolor(100, 100, 100)
        ),
        // Frame 3
        format!(
            "\n               \n     {}  \n     {} \n     {}  \n               ",
            "\\ *  * /".white(),
            "* POOF! *".white().bold(),
            "/ *  * \\".white()
        ),
        // Frame 4
        format!(
            "\n               \n        {}   \n      {} \n        {}   \n               ",
            ".  .".truecolor(150, 150, 150),
            ".      .".truecolor(150, 150, 150),
            ".  .".truecolor(150, 150, 150)
        ),
        // Frame 5
        format!(
            "\n               \n               \n               \n               \n               "
        ),
    ];

    for (i, frame) in frames.iter().enumerate() {
        print!("\x1B[2J\x1B[1;1H");

        if i < 2 {
            println!("Casting save spell...\n{}", frame);
            thread::sleep(Duration::from_millis(600));
        } else {
            println!("Abracadabra!\n{}", frame);
            thread::sleep(Duration::from_millis(300));
        }
        io::stdout().flush().unwrap();
    }
    print!("\x1B[2J\x1B[1;1H");
}

// Main function to run the to-do list application
fn main() {
    // Display the witch greeting animation
    witch_greeting_animation();

    // Create a new TodoList instance (this will load existing tasks from "tasks.json" if it exists)
    let mut todo_list = TodoList::new();

    // Main loop to display the menu and handle user input
    loop {
        println!("\n ### To-Do List ###");
        println!("1  Add Task");
        println!("2  List Tasks");
        println!("3  Complete Task");
        println!("4  Delete Task");
        println!("5  Exit");
        print!("Select an option number and press Enter: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("\nFailed to read input.");

        match choice.trim() {
            "1" => {
                println!("  (Tip: add !high or !low at the end of your task to set priority)");
                print!("Enter task description: ");
                io::stdout().flush().unwrap();
                let mut desc = String::new();
                io::stdin()
                    .read_line(&mut desc)
                    .expect("\nFailed to read input.");
                todo_list.add_task(desc.trim().to_string());
            }
            "2" => {
                todo_list.list_tasks();
            }
            "3" => {
                print!("Enter task number that you want to mark as complete: ");
                io::stdout().flush().unwrap();
                let mut index_str = String::new();
                io::stdin()
                    .read_line(&mut index_str)
                    .expect("\nFailed to read input.");
                if let Ok(index) = index_str.trim().parse::<usize>() {
                    todo_list.complete_task(index);
                } else {
                    println!("\nPlease enter a valid task number.");
                }
            }
            "4" => {
                print!("Enter the task number to delete: ");
                io::stdout().flush().unwrap();
                let mut index_str = String::new();
                io::stdin()
                    .read_line(&mut index_str)
                    .expect("\nFailed to read input.");
                if let Ok(index) = index_str.trim().parse::<usize>() {
                    todo_list.delete_task(index);
                } else {
                    println!("\nPlease enter a valid task number.");
                }
            }
            "5" => {
                // Display the witch ending animation before exiting
                witch_ending_animation();
                break;
            }
            _ => {
                println!("\nInvalid option, please try again.");
            }
        }
    }
}
