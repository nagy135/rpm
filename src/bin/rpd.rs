extern crate crypto;

// use std::env;
use std::fmt;
use std::fs;
// use std::io;
use std::fs::OpenOptions;
// use std::io::prelude::*;
// use std::time::SystemTime;
use self::crypto::digest::Digest;
use self::crypto::sha2::Sha256;
use std::str::from_utf8;

use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};

use rpm::constants;
use rpm::constants::Event as Event;


// {{{ RECORD IMPLEMENTATION

#[derive(Debug)]
struct Record {
    key     : String,
    login   : String,
    password: String
}

impl Record {
    fn save(&self) {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(constants::STORAGE)
            .unwrap();

        if let Err(e) = writeln!(file, "{}", &self) {
            panic!("Couldn't write to file: {}", e);
        }
        println!("Record was saved successfully:\n{}", &self);
    }
}
impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ||| {} ||| {}", self.key, self.login, self.password)
    }
}

// }}}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    let mut validated: bool = false;
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 3333");
    for stream in listener.incoming() {
        println!("before EVERYTHING: {}", validated);
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move|| {
                    // connection succeeded
                    handle_client(stream, &mut validated)
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    // close the socket server
    drop(listener);
}

fn handle_client(mut stream: TcpStream, validated: &mut bool) {
    let mut data = [0 as u8; 50]; // using 50 byte buffer
    match stream.read(&mut data) {
        Ok(size) => {
            let flag = &data[0];
            println!("validated before: {}", validated);
            let mut new_validated = false;
            let content = from_utf8(&data[1..size]).unwrap();
            let response: String = match Event::from(flag) {
                Event::New => handle_new(),
                Event::Get => handle_get(),
                Event::Validate => handle_validate(&content, &mut new_validated),
                Event::Verify => handle_verify()
            };
            *validated = new_validated;
            let response = response.into_bytes();
            println!("validated_after {}", new_validated);
            println!("flag {:?}", flag);
            println!("content {:?}", content);
            stream.write(&response).unwrap();
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
        }
    } {}
}
fn is_password_valid(pass: &str) -> Result<(), &str> {
    let mut hasher = Sha256::new();
    hasher.input_str(pass);
    let hash = hasher.result_str();
    println!("hash {}", hash);

    let old_hash = fs::read_to_string(constants::PASSWORD_HASH_HOLDER)
        .expect("Something went wrong reading the hash file");
    if old_hash.trim().is_empty() {
        return Err("Password is empty");
    }
    if hash != old_hash.trim() {
        return Err("Your password does not match !!!");
    }
    Ok(())
}

// {{{ handles

fn handle_validate(pass: &str, new_validated: &mut bool) -> String {

    println!("handle validate with pass {} {}", pass, new_validated);
    let validation_res = is_password_valid(pass);
    match validation_res {
        Ok(()) => {
            *new_validated = true;
            return "validated successfully".to_string()
        },
        Err(reason) => {
            *new_validated = false;
            return reason.to_string()
        }
    }
}

fn handle_verify() -> String {
    println!("handle verify");
    "verify".to_string()
}

fn handle_new() -> String {
    println!("handle new");
    "new".to_string()
}

fn handle_get() -> String {
    println!("handle get");
    "get".to_string()
}

// }}} handles
