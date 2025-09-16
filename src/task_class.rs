static mut CURR_NUM: u8 = 0;

#[derive(Debug)]
enum TaskStatus {
    NotDone,
    InProgress,
    Done,
}

pub struct Tasks {
    tasks: Vec<Task>,
}

#[derive(Debug)]
pub struct Task {
    pub id: u8,
    pub status: TaskStatus,
    pub describtion: String,
}

impl Tasks {
    pub fn new(tasks: Vec<Task>) -> Self {
        Tasks { tasks }
    }

    pub fn add_task(&mut self, desc: &str) {
        unsafe { CURR_NUM += 1 };

        self.tasks.push(Task {
            id: unsafe { CURR_NUM },
            status: TaskStatus::NotDone,
            describtion: desc.to_string(),
        })
    }

    pub fn update_task(&mut self, id: u8) {
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

    pub fn delete_task(&mut self, id: u8) {
        for i in 0..self.tasks.len() {
            if self.tasks[i].id == id {
                self.tasks.remove(i);
                break;
            }
        }
    }

    pub fn list_all(&self) {
        println!("{:#?}", self.tasks)
    }
}
