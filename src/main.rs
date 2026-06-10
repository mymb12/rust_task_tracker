use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};

use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use std::error::Error;

pub mod task_class;
use task_class::{TaskStatus, Tasks};
use uuid::Uuid;

type AppState = Arc<Mutex<Tasks>>;

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

    if args.len() > 1 && args[1] != "serve" {
        process_input(&args, &mut tasks).await;
        tasks.update_json(&filepath);

        return Ok(());
    }

    let state = Arc::new(Mutex::new(tasks));

    let app = Router::new()
        .route("/api/tasks", get(get_all_tasks))
        .route("/api/tasks", post(add_task))
        .route("/api/tasks/:id", put(update_task))
        .route("/api/tasks/:id", delete(delete_task))
        .nest_service("/", ServeDir::new("static"))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6570").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn get_all_tasks(State(state): State<AppState>) -> impl IntoResponse {
    let tasks = state.lock().await;
    Json(tasks.get_all_tasks())
}
async fn add_task(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut tasks = state.lock().await;

    let description = payload["description"].as_str().unwrap_or("Untitled Task");

    tasks.add_task(None, description, TaskStatus::NotDone).await;

    (StatusCode::CREATED, Json(tasks.get_all_tasks()))
}
async fn update_task(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let mut tasks = state.lock().await;

    match Uuid::parse_str(&id) {
        Ok(uuid) => {
            tasks.update_task(uuid).await;
            (StatusCode::OK, Json(tasks.get_all_tasks()))
        }
        Err(_) => (StatusCode::BAD_REQUEST, Json(vec![])),
    }
}
async fn delete_task(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let mut tasks = state.lock().await;

    match Uuid::parse_str(&id) {
        Ok(uuid) => {
            tasks.delete_task(uuid).await;
            (StatusCode::OK, Json(tasks.get_all_tasks()))
        }
        Err(_) => (StatusCode::BAD_REQUEST, Json(vec![])),
    }
}

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
