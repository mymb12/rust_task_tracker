use std::env;
use std::error::Error;
use std::fs;
use std::fs::OpenOptions;
use std::io::{self, Write};
use uuid::Uuid;

pub mod task_class;
use task_class::Tasks;

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

fn get_json_data(filepath: &String) -> serde_json::Value {
    check_file_existance(filepath);

    let res: Result<String, std::io::Error> = fs::read_to_string(filepath);
    let s = match res {
        Ok(s) => s,
        Err(_) => panic!("Can't read it"),
    };

    serde_json::from_str(&s).expect("Can't parse json")
}

fn update_json_data(json_data: &serde_json::Value, filepath: &String) {
    std::fs::write(filepath, serde_json::to_string_pretty(json_data).unwrap())
        .expect("Can't write file");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //TODO: create a postgreql implementation
    //TODO: test created script
    let args: Vec<String> = env::args().collect();

    let filepath = String::from("data.json");
    let mut json_data = get_json_data(&filepath);

    let mut array = json_data.as_array_mut();
    let mut tasks = Tasks::create_tasks_instance(&mut array);

    if let Err(e) = tasks.connect_database().await {
        eprintln!("Failed to connect to database: {}", e);
        println!("Continuing in JSON-only mode...");
    }

    process_input(&args, &mut tasks).await;

    let updated_json = tasks.to_json_value();
    update_json_data(&updated_json, &filepath);

    Ok(())
}
