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
        println!("Task added successfully.");
    }

    fn list_tasks(&self) {
        if self.tasks.is_empty() {
            println!("There are no tasks in the to-do list.");
            return;
        }
        for (index, task) in self.tasks.iter().enumerate() {
            let status = if task.completed { "|X|" } else { "| |" };
            println!("{} {} {}", index + 1, status, task.description);
        }
    }

    fn complete_task(&mut self, index: usize) {
        if index == 0 || index > self.tasks.len() {
            println!("Invalid task number.");
            return;
        }
        self.tasks[index - 1].completed = true;
        println!("Task {} marked as completed.", index);
    }
}
