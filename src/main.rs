use std::env;

use std::fs;

pub mod task_class;

use task_class::Tasks;

fn process_input(args: &[String], tasks: &mut Tasks, json_data: &mut serde_json::Value) {
    //TODO: rethink the way of using json_data
    let command = args[1].clone();

    match command.as_str() {
        "add" => tasks.add_task(args[2].clone().as_str()),
        "update" => tasks.update_task(
            args[2]
                .clone()
                .parse::<u8>()
                .expect("second argument cannot be converted to u8"),
        ),
        "remove" => tasks.delete_task(
            args[2]
                .clone()
                .parse::<u8>()
                .expect("second argument cannot be converted to u8"),
        ),
        _ => println!("such command was not defined"),
    };
}

fn get_json_data() -> serde_json::Value {
    let res: Result<String, std::io::Error> = fs::read_to_string("data.json");
    let s = match res {
        Ok(s) => s,
        Err(_) => panic!("Can't read it"),
    };

    serde_json::from_str(&s).expect("Can't parse json")
}

fn update_json_data(json_data: &serde_json::Value) {
    std::fs::write(
        "data.json",
        serde_json::to_string_pretty(json_data).unwrap(),
    )
    .expect("Can't write file");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut tasks = Tasks::new(Vec::new());

    let mut json_data = get_json_data();

    process_input(&args, &mut tasks, &mut json_data);

    println!("{:#}", json_data);

    update_json_data(&json_data);
}
