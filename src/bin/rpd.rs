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

use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};

use rpm::constants;

#[derive(Debug)]
struct Record {
    key     : String,
    login   : String,
    password: String
}

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

fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8; 50]; // using 50 byte buffer
    match stream.read(&mut data) {
        Ok(size) => {
            // echo everything!
            println!("{:?}", &data[0..size]);
            stream.write(&data[0..size]).unwrap();
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
        }
    } {}
}

// {{{ RECORD IMPLEMENTATION
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
