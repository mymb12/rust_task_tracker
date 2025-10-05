use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

use sqlx::postgres::PgPool;
use std::error::Error;

use std::fs::{self, OpenOptions};
use std::io::{self, Write};

#[derive(Debug, Clone, serde::Serialize)]
pub enum TaskStatus {
    NotDone,
    InProgress,
    Done,
}

pub struct Tasks {
    tasks: Vec<Task>,
    database: Option<PgPool>,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: Uuid,
    pub status: TaskStatus,
    pub describtion: String,
    pub time_created: chrono::DateTime<Utc>,
    pub time_updated: Option<chrono::DateTime<Utc>>,
}

impl TaskStatus {
    pub fn to_string(&self) -> String {
        match self {
            TaskStatus::NotDone => "NotDone".to_string(),
            TaskStatus::InProgress => "InProgress".to_string(),
            TaskStatus::Done => "Done".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "notdone" => TaskStatus::NotDone,
            "inprogress" => TaskStatus::InProgress,
            "done" => TaskStatus::Done,
            _ => TaskStatus::NotDone,
        }
    }
}

impl Tasks {
    pub fn new(tasks: Vec<Task>) -> Self {
        Tasks {
            tasks,
            database: None,
        }
    }

    pub async fn connect_database(&mut self) -> Result<(), Box<dyn Error>> {
        let url = "postgres://postgres:123@localhost:5433/rust_task_tracker";
        let pool = sqlx::postgres::PgPool::connect(url).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS task (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                status VARCHAR(20) NOT NULL DEFAULT 'NotDone',
                describtion TEXT NOT NULL,
                time_created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                time_updated TIMESTAMPTZ
            )",
        )
        .execute(&pool)
        .await?;

        self.database = Some(pool);
        println!("Postgres database connected");
        Ok(())
    }

    pub async fn add_task(&mut self, id: Option<Uuid>, desc: &str, status: TaskStatus) {
        let new_task = Task {
            id: id.unwrap_or(Uuid::new_v4()),
            status,
            describtion: desc.to_string(),
            time_created: Utc::now(),
            time_updated: None,
        };

        if let Some(pool) = &self.database {
            let result = sqlx::query(
                "INSERT INTO task (id, status, describtion, time_created, time_updated) 
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(new_task.id)
            .bind(new_task.status.to_string())
            .bind(&new_task.describtion)
            .bind(new_task.time_created)
            .bind(new_task.time_updated)
            .execute(pool)
            .await;

            match result {
                Ok(_) => println!("Task added to database with ID: {}", new_task.id),
                Err(e) => eprintln!("Failed to add task to database: {}", e),
            }
        }

        self.tasks.push(new_task);
    }

    fn check_file_existance(filepath: &String) {
        let file_exists = OpenOptions::new()
            .write(true)
            .create_new(true)
            .truncate(false)
            .open(filepath);

        match file_exists {
            Ok(mut f) => {
                let _ = f.write_all(b"[]");
                println!("{} file wasn't found, so new file was created", filepath);
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                println!("{} file already exists", filepath)
            }
            Err(e) => {
                eprintln!("Error creating file: {}", e)
            }
        }
    }

    pub fn get_json_data(&mut self, filepath: &String) {
        Self::check_file_existance(filepath);

        let res: Result<String, std::io::Error> = fs::read_to_string(filepath);
        let s = match res {
            Ok(s) => s,
            Err(_) => panic!("Can't read it"),
        };

        let mut array: serde_json::Value = serde_json::from_str(&s).expect("Can't parse json");
        let mut array = array.as_array_mut();

        self.tasks = Self::create_tasks_instance(&mut array).tasks;
    }

    pub fn update_json(&mut self, filepath: &String) {
        let updated_json = self.to_json_value();

        std::fs::write(
            filepath,
            serde_json::to_string_pretty(&updated_json).unwrap(),
        )
        .expect("Can't write file");
    }

    pub fn inset_object_from_json(
        &mut self,
        id: Uuid,
        desc: &str,
        status: Option<&str>,
        time_created: &str,
    ) {
        let stat = TaskStatus::from_string(status.unwrap_or("NotDone"));

        self.tasks.push(Task {
            id,
            status: stat,
            describtion: desc.to_string(),
            time_created: time_created.parse().unwrap(),
            time_updated: Some(Utc::now()),
        })
    }

    pub async fn update_task(&mut self, id: Uuid) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.status = match task.status {
                TaskStatus::NotDone => TaskStatus::InProgress,
                TaskStatus::InProgress => TaskStatus::Done,
                TaskStatus::Done => TaskStatus::Done, // Stay Done
            };
            task.time_updated = Some(Utc::now());

            if let Some(pool) = &self.database {
                let result =
                    sqlx::query("UPDATE task SET status = $1, time_updated = $2 WHERE id = $3")
                        .bind(task.status.to_string())
                        .bind(task.time_updated)
                        .bind(id)
                        .execute(pool)
                        .await;

                match result {
                    Ok(rows) => {
                        if rows.rows_affected() > 0 {
                            println!("Task {} updated in database", id);
                        } else {
                            println!("Task {} not found in database", id);
                        }
                    }
                    Err(e) => eprintln!("Failed to update task in database: {}", e),
                }
            }
        } else {
            println!("Task {} not found", id);
        }
    }

    pub async fn delete_task(&mut self, id: Uuid) {
        if let Some(pos) = self.tasks.iter().position(|t| t.id == id) {
            self.tasks.remove(pos);
            println!("Task {} deleted locally", id);

            if let Some(pool) = &self.database {
                let result = sqlx::query("DELETE FROM task WHERE id = $1")
                    .bind(id)
                    .execute(pool)
                    .await;

                match result {
                    Ok(rows) => {
                        if rows.rows_affected() > 0 {
                            println!("Task {} deleted from database", id);
                        } else {
                            println!("Task {} not found in database", id);
                        }
                    }
                    Err(e) => eprintln!("Failed to delete task from database: {}", e),
                }
            }
        } else {
            println!("Task {} not found", id);
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
        println!("{:#?}", self.tasks);
    }

    pub fn get_all_tasks(&self) -> Vec<serde_json::Value> {
        self.tasks
            .iter()
            .map(|task| {
                serde_json::json!({
                    "id": task.id,
                    "describtion": task.describtion,
                    "status": task.status,
                    "time_created": task.time_created,
                    "time_updated": task.time_updated,
                })
            })
            .collect()
    }

    pub fn find_task(&mut self, id: &Uuid) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == *id)
    }
}
