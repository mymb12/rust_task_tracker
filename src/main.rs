use std::env;
use std::error::Error;
use uuid::Uuid;

pub mod task_class;
use task_class::Tasks;

async fn process_input(args: &[String], tasks: &mut Tasks) {
    if args.len() < 2 {
        tasks.list_all();
        return;
    }
    let command = args[1].clone();
    match command.as_str() {
        "add" => {
            tasks
                .add_task(
                    None,
                    args[2].clone().as_str(),
                    task_class::TaskStatus::NotDone,
                )
                .await;
        }
        "update" => {
            match Uuid::parse_str(&args[2]) {
                Ok(uuid) => tasks.update_task(uuid).await,
                Err(_) => println!("Invalid UUID format: {}", args[2]),
            };
        }
        "remove" => match Uuid::parse_str(&args[2]) {
            Ok(uuid) => tasks.delete_task(uuid).await,
            Err(_) => println!("Invalid UUID format: {}", args[2]),
        },
        _ => println!("such command was not defined"),
    };

    tasks.list_all();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let filepath = String::from("data.json");
    let mut tasks = Tasks::new(Vec::new());
    tasks.get_json_data(&filepath);

    if let Err(e) = tasks.connect_database().await {
        eprintln!("Failed to connect to database: {}", e);
        println!("Continuing in JSON-only mode...");
    }

    process_input(&args, &mut tasks).await;
    tasks.update_json(&filepath);

    Ok(())
}
/*
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TaskClass {
    body: String,
    id: Option<String>,
}

#[tokio::main]
async fn main() {
    let router01 = Router::new().route("/tasks", get(list_all).post(add_task));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:6570").await.unwrap();

    axum::serve(listener, router01).await.unwrap();
}

#[debug_handler]
async fn list_all() -> Json<TaskClass> {
    Json::from(TaskClass {
        body: "some".to_string(),
        id: Some(uuid::Uuid::new_v4().to_string()),
    })
}

async fn add_task(Query(mut v): Query<TaskClass>) -> Json<TaskClass> {
    v.id = Some(uuid::Uuid::new_v4().to_string());

    Json::from(v)
} */
