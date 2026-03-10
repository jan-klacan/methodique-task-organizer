use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};

#[derive(Serialize, Deserialize)]
struct Task {
    description: String,
    completed: bool,
}

impl Task {
    fn new(description: String) -> Task {
        Task {
            description,
            completed: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TodoList {
    tasks: Vec<Task>,
}

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

    fn add_task(&mut self, description: String) {
        self.tasks.push(Task::new(description));
        self.save_to_file(); // Save the updated list to a file
        println!("\nTask added successfully.");
    }

    fn delete_task(&mut self, index: usize) {
        if index == 0 || index > self.tasks.len() {
            println!("\nInvalid task number.");
            return;
        }
        self.tasks.remove(index - 1);
        self.save_to_file();
        println!("Task deleted successfully.");
    }

    fn list_tasks(&self) {
        if self.tasks.is_empty() {
            println!("\nThere are no tasks in the to-do list.");
            return;
        }
        for (index, task) in self.tasks.iter().enumerate() {
            let status = if task.completed { "|X|" } else { "| |" };
            println!("{} {} {}", index + 1, status, task.description);
        }
    }

    fn complete_task(&mut self, index: usize) {
        if index == 0 || index > self.tasks.len() {
            println!("\nInvalid task number.");
            return;
        }
        self.tasks[index - 1].completed = true;
        self.save_to_file(); // Save the updated list to a file
        println!("\nTask {} marked as completed.", index);
    }
}

fn main() {
    let mut todo_list = TodoList::new();
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
                println!("\nExiting the application. See ya!");
                break;
            }
            _ => {
                println!("\nInvalid option, please try again.");
            }
        }
    }
}
