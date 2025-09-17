use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize)]
pub enum TaskStatus {
    NotDone,
    InProgress,
    Done,
}

pub struct Tasks {
    tasks: Vec<Task>,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: Uuid,
    pub status: TaskStatus,
    pub describtion: String,
    pub time_created: chrono::DateTime<Utc>,
    pub time_updated: Option<chrono::DateTime<Utc>>,
}

impl Tasks {
    pub fn new(tasks: Vec<Task>) -> Self {
        Tasks { tasks }
    }

    pub fn add_task(&mut self, id: Option<Uuid>, desc: &str, status: TaskStatus) {
        self.tasks.push(Task {
            id: id.unwrap_or(Uuid::new_v4()),
            status,
            describtion: desc.to_string(),
            time_created: Utc::now(),
            time_updated: None,
        })
    }

    pub fn inset_object_from_json(
        &mut self,
        id: Uuid,
        desc: &str,
        status: Option<&str>,
        time_created: &str,
    ) {
        let status = match status.unwrap_or("").trim().to_lowercase().as_str() {
            "notdone" => TaskStatus::NotDone,
            "inprogress" => TaskStatus::InProgress,
            "done" => TaskStatus::Done,
            _ => TaskStatus::NotDone,
        };

        self.tasks.push(Task {
            id,
            status,
            describtion: desc.to_string(),
            time_created: time_created.parse().unwrap(),
            time_updated: Some(Utc::now()),
        })
    }

    pub fn update_task(&mut self, id: Uuid) {
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

                self.tasks[i].time_updated = Some(Utc::now());

                break;
            }
        }
    }

    pub fn delete_task(&mut self, id: Uuid) {
        for i in 0..self.tasks.len() {
            if self.tasks[i].id == id {
                self.tasks.remove(i);
                break;
            }
        }
    }

    pub fn create_tasks_instance(array: &mut Option<&mut Vec<Value>>) -> Tasks {
        match array {
            Some(array) => {
                let mut tasks = Tasks::new(Vec::new());

                for i in 0..array.len() {
                    tasks.inset_object_from_json(
                        Uuid::parse_str(array[i]["id"].as_str().unwrap()).unwrap(),
                        array[i]["describtion"].as_str().unwrap(),
                        Some(array[i]["status"].as_str().unwrap()),
                        array[i]["time_created"]
                            .as_str()
                            .unwrap_or("1999-01-01T00:00:00Z"),
                    );
                }

                tasks
            }
            None => Tasks::new(Vec::new()),
        }
    }

    pub fn to_json_value(&self) -> Value {
        let task_values: Vec<Value> = self
            .tasks
            .iter()
            .map(|task| {
                json!({
                    "id": task.id,
                    "describtion": task.describtion,
                    "status": task.status,
                    "time_created": task.time_created,
                    "time_updated": task.time_updated,
                })
            })
            .collect();

        Value::Array(task_values)
    }

    pub fn list_all(&self) {
        println!("{:#?}", self.tasks)
    }
}
