use colored::Colorize;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};
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

// Define a TaskData struct that contains a vector of Task structs and derive Serialize and Deserialize for JSON handling; this struct will be used to read/write the list of tasks to/from a JSON file
#[derive(Serialize, Deserialize)]
struct TaskData {
    tasks: Vec<Task>,
}

// Define a TodoList struct that contains a TaskData struct for managing the list of tasks and a SentenceEmbeddingsModel for performing semantic similarity checks when adding new tasks
struct TodoList {
    data: TaskData,                 // Contains the list of tasks
    embedder: SentenceEmbeddingsModel, // Pre-trained sentence embedding model for semantic similarity checks
}

// Implement methods for TodoList
impl TodoList {
    fn new() -> TodoList {
        // Print a colored message to the user indicating that the PyTorch model is being loaded
        println!(
            "{}",
            "Loading PyTorch model ... (this may take a moment)".cyan()
        );

        // Use the rust-bert library to load a pre-trained sentence embedding model from Hugging Face's model hub
        let embedder =
            SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2)
                .create_model()
                .expect("Failed to load ML model");

        // Attempt to read existing tasks from "tasks.json" and parse it into a TaskData struct; if the file does not exist or cannot be parsed, initialize with an empty task list
        let mut data = TaskData { tasks: Vec::new() };
        if let Ok(json_data) = fs::read_to_string("tasks.json") {
            if let Ok(parsed_data) = serde_json::from_str::<TaskData>(&json_data) {
                data = parsed_data;
            }
        }

        TodoList { data, embedder } // Return a new instance of TodoList with the loaded data and embedder
    }

    // Helper method to save the file
    fn save_to_file(&self) {
        // Convert the TodoList struct into a JSON string
        let json = serde_json::to_string_pretty(&self.data).expect("Failed to format JSON.");
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

        // ----- ML SEMANTIC SIMILARITY CHECK -----
        if !self.data.tasks.is_empty() {
            // Encode the NEW task into a vector
            let new_embedding = &self.embedder.encode(&[&description]).unwrap()[0];

            for existing_task in &self.data.tasks {
                if existing_task.completed {
                    continue;
                } // Skip completed tasks

                // Encode the EXISTING task into a vector
                let existing_embedding =
                    &self.embedder.encode(&[&existing_task.description]).unwrap()[0];

                // Calculate the cosine similarity between the new task and the existing task embeddings
                let similarity = cosine_similarity(new_embedding, existing_embedding);

                // If semantic match is > 80%, warn the user
                if similarity > 0.80 {
                    println!("\n{}", "Wait a second!".red().bold());
                    println!(
                        "This looks very similar to an existing task: {}",
                        existing_task.description.yellow()
                    );
                    println!("(Semantic Match: {:.1}%)\n", similarity * 100.0);

                    print!("Do you still want to add it? (y/n): ");
                    io::stdout().flush().unwrap();
                    let mut confirm = String::new();
                    io::stdin().read_line(&mut confirm).unwrap();

                    // If the user does not confirm with "y", cancel adding the task
                    if !confirm.trim().eq_ignore_ascii_case("y") {
                        println!("Task cancelled.");
                        return; // Exit function without saving
                    }
                    break;
                }
            }
        }
        // --------------------------------------

        self.data.tasks.push(Task::new(description, priority)); // Add the new task to the list
        self.data.tasks.sort(); // Sort the tasks after adding a new one
        self.save_to_file(); // Save the updated list to a file
        println!("\nTask added successfully.");
    }

    // Delete a task by its index (1-based) and save the updated list to a file
    fn delete_task(&mut self, index: usize) {
        if index == 0 || index > self.data.tasks.len() {
            println!("\nInvalid task number.");
            return;
        }
        self.data.tasks.remove(index - 1);
        self.save_to_file();
        println!("Task deleted successfully.");
    }

    // List all tasks with their status (completed or not)
    fn list_tasks(&self) {
        if self.data.tasks.is_empty() {
            println!("\nThere are no tasks in the to-do list.");
            return;
        }

        // Iterate over the tasks and print their 1-based index, status, description, and priority with colors
        for (index, task) in self.data.tasks.iter().enumerate() {
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
        if index == 0 || index > self.data.tasks.len() {
            println!("\nInvalid task number.");
            return;
        }
        self.data.tasks[index - 1].completed = true;
        self.data.tasks.sort(); // Sort the tasks after marking one as completed
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
