use std::io::{self, Write};

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

struct TodoList {
    tasks: Vec<Task>,
}

impl TodoList {
    fn new() -> TodoList {
        TodoList { tasks: Vec::new() }
    }

    fn add_task(&mut self, description: String) {
        self.tasks.push(Task::new(description));
        println!("\nTask added successfully.");
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
        println!("4  Exit");
        print!("Enter the option number: ");
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
                println!("\nExiting the application. See ya!");
                break;
            }
            _ => {
                println!("\nInvalid option, please try again.");
            }
        }
    }
}
