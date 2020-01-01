extern crate crypto;

// use std::env;
use std::fmt;
// use std::fs;
// use std::io;
use std::fs::OpenOptions;
// use std::io::prelude::*;
// use std::time::SystemTime;
// use self::crypto::digest::Digest;
// use self::crypto::sha2::Sha256;
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
    run_server();
}

fn run_server() {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 3333");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move|| {
                    // connection succeeded
                    handle_client(stream)
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


fn handle_validate() -> String {
    println!("handle validate");
    "validate".to_string()
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

fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8; 50]; // using 50 byte buffer
    match stream.read(&mut data) {
        Ok(size) => {
            let flag = &data[0];
            let response: String = match Event::from(flag) {
                Event::New => handle_new(),
                Event::Get => handle_get(),
                Event::Validate => handle_validate(),
                Event::Verify => handle_verify()
            };
            let data = &data[1..size];
            let response = response.into_bytes();
            // println!("{:?}", &data[0..size]);
            println!("flag {:?}", flag);
            println!("data {:?}", from_utf8(&data).unwrap());
            stream.write(&response).unwrap();
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
        }
    } {}
}
