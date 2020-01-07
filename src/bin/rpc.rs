extern crate rpassword;

use rpassword::read_password;
use std::env;
use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;

use rpm::constants::Event as Event;

fn main(){
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let command: &String = &args[1];
        match command.as_ref() {
            "new" => new(&args),
            "get" => get(&args),
            "validate" => validate(),
            "change" => change(),
            "list" => list(),
            "--help" => help(),
            _ => println!("Unknown command...try --help")
        };
    }
}

fn first_zero(data: &[u8; 50]) -> usize {
    for i in 1..50 { // skipping first index, might be validation zero
        if data[i] == 0 {
            return i
        }
    }
    50
}

fn send_to_daemon(message: String, event: Event) -> Result<String, String>{
    let result: Result<String, String>;
    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {
            let mut msg_vec = message.into_bytes();
            msg_vec.insert(0, event.to_u8());
            let msg: &[u8] = &msg_vec; // c: &[u8]
            stream.write(msg).unwrap();

            let mut data = [0 as u8; 50];
            match stream.read(&mut data) {
                Ok(_) => {
                    let zero_index = first_zero(&data);
                    let validated = &data[0];
                    let text = from_utf8(&data[1..zero_index]).unwrap().to_string();
                    if *validated == 0 {
                        result = Err("Validation failed!!!".to_string());
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
    println!("args {:?}", args);
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

fn new(args: &Vec<String>){
    println!("Creating new record");
    if args.len() < 4 {
        panic!("Need at least 4 arguments !!! new, key, password (or new, key, login, password)");
    }
    let response = send_to_daemon(concat_vec(&args), Event::New);
    println!("response inside client new {:?}", response);
}

fn get(args: &Vec<String>){
    println!("Getting record from given key {:?}", args);
    if args.len() < 3 {
        panic!("Need at least 3 arguments !!! get, key (returns password, login returned by --login or -l)");
    }
    let response = send_to_daemon(concat_vec(&args), Event::Get);
    println!("response inside client get {:?}", response);
}
fn validate(){
    println!("Validating...");
    println!("Type master password: ");
    let password = read_password().unwrap();
    let response = send_to_daemon(password, Event::Validate);
    println!("response inside client validate {:?}", response);
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
