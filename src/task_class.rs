static mut CURR_NUM: u64 = 0;

#[derive(Debug)]
pub enum TaskStatus {
    NotDone,
    InProgress,
    Done,
}

pub struct Tasks {
    tasks: Vec<Task>,
}

#[derive(Debug)]
pub struct Task {
    pub id: u64,
    pub status: TaskStatus,
    pub describtion: String,
}

impl Tasks {
    pub fn new(tasks: Vec<Task>) -> Self {
        Tasks { tasks }
    }

    pub fn add_task(&mut self, id: Option<u64>, desc: &str, status: Option<&str>) {
        unsafe { CURR_NUM += 1 };

        let status = match status.unwrap_or("").trim().to_lowercase().as_str() {
            "notdone" => TaskStatus::NotDone,
            "inprogress" => TaskStatus::InProgress,
            "done" => TaskStatus::Done,
            _ => TaskStatus::NotDone,
        };

        self.tasks.push(Task {
            id: id.unwrap_or(unsafe { CURR_NUM }),
            status,
            describtion: desc.to_string(),
        })
    }

    pub fn update_task(&mut self, id: u64) {
        for i in 0..self.tasks.len() {
            if self.tasks[i].id == id {
                match self.tasks[i].status {
                    TaskStatus::NotDone => {
                        self.tasks[i].status = TaskStatus::InProgress;
                    }
                    TaskStatus::InProgress => {
                        self.tasks[i].status = TaskStatus::Done;
                    }
                    TaskStatus::Done => {}
                }

                break;
            }
        }
    }

    pub fn delete_task(&mut self, id: u64) {
        for i in 0..self.tasks.len() {
            if self.tasks[i].id == id {
                self.tasks.remove(i);
                break;
            }
        }
    }

    pub fn create_tasks_instance(array: &mut Option<&mut Vec<serde_json::Value>>) -> Tasks {
        match array {
            Some(array) => {
                let mut tasks = Tasks::new(Vec::new());

                for i in 0..array.len() {
                    tasks.add_task(
                        array[i]["id"].as_u64(),
                        array[i]["desc"].as_str().unwrap(),
                        Some(array[i]["status"].as_str().unwrap()),
                    );
                }

                tasks
            }
            None => Tasks::new(Vec::new()),
        }
    }

    pub fn list_all(&self) {
        println!("{:#?}", self.tasks)
    }
}
