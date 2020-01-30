extern crate rpassword;


use rpassword::read_password;
use std::env;
use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;

use rpm::constants::Event as Event;

fn main() {
    std::process::exit(match run_app() {
        Ok(output) => {
            print!("{}", output);
            0
        }
        Err(err) => {
            eprintln!("error: {:?}", err);
            1
        }
    });
}

fn run_app() -> Result<String, String>{
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let command: &String = &args[1];
        let response = match command.as_ref() {
            "new" => new(&args),
            "get" => get(&args),
            "delete" => delete(&args),
            "validate" => validate(&args),
            "changeMP" => change_mp(),
            "change" => change(&args),
            "init" => init(),
            "list" => list(),
            "--help" => help(),
            _ => Err("Unknown command...try --help".to_string())
        };
        return response;
    }
    Err(String::from("No command specified...try --help"))
}

fn first_zero(data: &[u8; 200]) -> usize {
    for i in 1..200 { // skipping first index, might be validation zero
        if data[i] == 0 {
            return i
        }
    }
    200
}

fn send_to_daemon(message: String, event: Event) -> Result<String, String>{
    let result: Result<String, String>;
    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {
            let mut msg_vec = message.into_bytes();
            msg_vec.insert(0, event.to_u8());
            let msg: &[u8] = &msg_vec; // c: &[u8]
            stream.write(msg).unwrap();

            let mut data = [0 as u8; 200];
            match stream.read(&mut data) {
                Ok(_) => {
                    let zero_index = first_zero(&data);
                    let validated = &data[0];
                    let text = from_utf8(&data[1..zero_index]).unwrap().to_string();
                    if *validated == 0 {
                        match event {
                            Event::Init => result = Ok(text.clone()),
                            _           => result = Err("Validation failed! use validate command".to_string())
                        }
                    } else {
                        result = Ok(text.clone());
                    }
                },
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                    result = Err("Read failed".to_string());
                }
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
            result = Err("Connect failed".to_string());
        }
    }
    result
}

// {{{ utils

fn concat_vec(args: &Vec<String>) -> String {
    let mut result = String::new();
    for (i,arg) in args.iter().enumerate() {
        if i > 1 {
            result.push_str(arg);
            if i < args.len() - 1 {
                result.push_str("#@#");
            }
        }
    }
    result
}

// }}} utils

fn new(args: &Vec<String>) -> Result<String, String>{
    if args.len() < 4 {
        return Err("Need at least 4 arguments !!! new, key, password (or new, key, login, password)".to_string());
    }
    let response = send_to_daemon(concat_vec(&args), Event::New);
    response
}

fn get(args: &Vec<String>) -> Result<String, String>{
    if args.len() < 3 {
        return Err("Need at least 3 arguments !!! get, key (returns password, login returned by -l flag)".to_string());
    }
    let response = send_to_daemon(concat_vec(&args), Event::Get);
    response
}

fn delete(args: &Vec<String>) -> Result<String, String>{
    if args.len() < 3 {
        return Err("Need 2 arguments, delete and key of record which we try to delete".to_string());
    }
    let response = send_to_daemon(concat_vec(&args), Event::Delete);
    response
}
fn validate(args: &Vec<String>) -> Result<String, String>{
    let password;
    if args.len() < 3 {
        println!("Type master password: ");
        password = read_password().unwrap();
    } else {
        password = args[2].clone();
    }
    let response = send_to_daemon(password, Event::Validate);
    response
}
fn init() -> Result<String, String>{
    println!("Type INITIAL master password (removes all records from storage): ");
    let init_password = read_password().unwrap();
    let response = send_to_daemon(init_password, Event::Init);
    response
}

fn change_mp() -> Result<String, String>{
    println!("Type NEW master password: ");
    let new_password = read_password().unwrap();
    let response = send_to_daemon(new_password, Event::ChangeMP);
    response
}
fn change(args: &Vec<String>) -> Result<String, String>{
    if args.len() < 4 {
        return Err("Need at least 4 arguments !!! change, key, password (or new, key, login, password)".to_string());
    }
    let response = send_to_daemon(concat_vec(&args), Event::Change);
    response
}
fn list() -> Result<String, String>{
    let response = send_to_daemon("".to_string(),  Event::List);
    response
}

fn help() -> Result<String, String>{
    println!("Rusty password daemon - password manager written in rust running in background

usage: rpc [command] [args...] [--help]
commands: [new, get, validate, change, list]
    new          - checks validation, prompts for key then password and saves hashed
    get [-l] key - checks validation, returns unencrypted password (or login with -l)
    validate     - prompts for master password and unlocks for PASS_DELAY seconds
    changeMP     - prompts for old master pass and then new twice
    list         - prints all keys for rofi integration
    ");
    Ok(String::new())
}
