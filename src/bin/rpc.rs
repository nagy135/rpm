use std::env;

fn main(){
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let command: &String = &args[1];
        match command.as_ref() {
            "new" => new(),
            "get" => get(&args),
            "validate" => validate(),
            "change" => change(),
            "list" => list(),
            "--help" => help(),
            _ => println!("Unknown command...try --help")
        };
    }
}


fn new(){
    println!("Creating new record");
}

fn get(args: &Vec<String>){
    println!("Getting record from given key {:?}", args);
}
fn validate(){
    println!("Validating...");
}
fn change(){
    println!("Changing password");
}
fn list(){
    println!("Return all keys each on one line");
}

fn help(){
    println!("Rusty password daemon - password manager written in rust running in background

usage: rpc [command] [args...] [--help]
commands: [new, get, validate, change, list]
    new          - checks validation, prompts for key then password and saves hashed
    get [-l] key - checks validation, returns unencrypted password (or login with -l)
    validate     - prompts for master password and unlocks for PASS_DELAY seconds
    change       - prompts for old master pass and then new twice
    list         - prints all keys for rofi integration
    ");
}
