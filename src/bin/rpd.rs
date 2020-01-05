extern crate crypto;

// use std::env;
use std::sync::{Mutex, Arc};
use std::time::Duration;
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
    // let mut validated: bool = false;
    let validated = Arc::new(Mutex::new(false));
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 3333");
    for stream in listener.incoming() {
        println!("========================================");
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                let handler;
                {
                    let validated_clone = Arc::clone(&validated);
                    handler = thread::spawn(move|| {
                        let mut validation_a = validated_clone.lock().unwrap();
                        println!("validation_a {}", validation_a);
                        client(stream, &mut validation_a)
                    });
                }
                {
                    if handler.join().unwrap() {
                        let validated_clone = Arc::clone(&validated);
                        thread::spawn(move|| {
                            println!("Validation successfull: starting timeout");
                            thread::sleep(Duration::from_secs(constants::PASS_DELAY));
                            let mut validation_a = validated_clone.lock().unwrap();
                            *validation_a = false;
                            println!("Authentication timeouted: rpm locked");
                        });
                    }
                }
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

fn client(mut stream: TcpStream, mut validate: &mut bool) -> bool {
    let mut data = [0 as u8; 50]; // using 50 byte buffer
    match stream.read(&mut data) {
        Ok(size) => {
            let flag = &data[0];
            let content = from_utf8(&data[1..size]).unwrap();
            let response: String = match Event::from(flag) {
                Event::New => handle_new(&mut validate),
                Event::Get => handle_get(&mut validate),
                Event::Validate => handle_validate(&content, &mut validate)
            };
            let response = response.into_bytes();
            println!("response {:?}", response);
            let mut chained_response = vec![];
            chained_response.push(*validate as u8);
            for i in response.iter(){
                chained_response.push(*i as u8);

            }
            let mut boxed: Box<[u8]> = chained_response.into_boxed_slice();
            println!("boxed {:?}", boxed);
            println!("flag {:?}", flag);
            println!("content {:?}", content);
            stream.write(&boxed).unwrap();
            *validate
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    }
}

// {{{ utils
fn is_password_valid(pass: &str) -> Result<(), &str> {
    let mut hasher = Sha256::new();
    hasher.input_str(pass);
    let hash = hasher.result_str();
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

// }}} utils

// {{{ handles

fn handle_validate(pass: &str, validate: &mut bool) -> String {
    let validation_res = is_password_valid(pass);
    match validation_res {
        Ok(()) => {
            *validate = true;
            "validated".to_string()
        },
        Err(reason) => {
            *validate = false;
            println!("reason {}", reason);
            "validation failed: password invalid".to_string()
        }
    }
}

fn handle_new(validated: &mut bool) -> String {
    if *validated {
        return "All nice".to_string();
    } else {
        return "FUUUUUUUUUUUUCK nice".to_string();
    }

    println!("handle new");
    "new".to_string()
}

fn handle_get(validated: &mut bool) -> String {
    if *validated {
        return "All nice".to_string();
    } else {
        return "FUUUUUUUUUUUUCK nice".to_string();
    }
    println!("handle get");
    "get".to_string()
}

// }}} handles
