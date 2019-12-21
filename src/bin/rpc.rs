use std::env;
use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;

// {{{ Enum Event
#[derive(Debug, Copy, Clone)]
enum Event {
    New = 1,
    Get = 2
}

fn enum_to_u8(e: &Event) -> u8 {
    *e as u8
}
// }}}

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

fn send_to_daemon(message: String, event: Event) -> String{
    let mut response = String::from("");
    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {
            let mut msg_vec = message.into_bytes();
            // println!("Sending : {:?}", msg);
            // println!("Sending event: {:?}", enum_to_u8(&event));
            msg_vec.insert(0, enum_to_u8(&event));
            println!("{:?}", msg_vec);
            let msg: &[u8] = &msg_vec; // c: &[u8]
            // let msg = b"haha";

            stream.write(msg).unwrap();

            let mut data = [0 as u8; 8];
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    let text = from_utf8(&data).unwrap().to_string();
                    println!("Response from daemon: {:?}", text);
                    response = text.clone();
                },
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    response
}

fn new(){
    println!("Creating new record");
    let response = send_to_daemon(String::from("testtest"), Event::New);
    println!("{:?}", response);
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
