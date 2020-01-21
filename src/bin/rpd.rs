extern crate crypto;

use openssl::symm::{decrypt, encrypt, Cipher};
use std::sync::{Mutex, Arc};
use std::time::Duration;
use std::fmt;
use std::fs;
use std::env;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use self::crypto::digest::Digest;
use self::crypto::sha2::Sha256;
use std::str::from_utf8;
use std::thread;
use std::path::Path;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};

use rpm::constants;
use rpm::constants::Event as Event;
use rpm::constants::Reason as Reason;


// {{{ RECORD IMPLEMENTATION

#[derive(Debug)]
struct Record {
    key     : String,
    data: String
}

impl Record {
    fn save(&self) {
        let root = env::var("XDG_CONFIG_HOME").unwrap();
        let path = Path::new(root.as_str());
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(path.join(constants::STORAGE))
            .unwrap();

        if let Err(e) = writeln!(file, "{}", &self) {
            panic!("Couldn't write to file: {}", e);
        }
        println!("Record was saved successfully:\n{}", &self);
    }
}
impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ||| {}", self.key, self.data)
    }
}

// }}}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    let validated = Arc::new(Mutex::new(false));
    let valid_password = Arc::new(Mutex::new(String::new()));

    println!("Server listening on port 3333");
    for stream in listener.incoming() {
        println!("========================================");
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                let handler;
                {
                    let validated_clone = Arc::clone(&validated);
                    let valid_password_clone = Arc::clone(&valid_password);
                    handler = thread::spawn(move|| {
                        let mut validation_a = validated_clone.lock().unwrap();
                        let mut valid_password_a = valid_password_clone.lock().unwrap();
                        client(stream, &mut validation_a, &mut valid_password_a)
                    });
                }
                {
                    if handler.join().unwrap() {
                        let validated_clone = Arc::clone(&validated);
                        let valid_password_clone = Arc::clone(&valid_password);
                        thread::spawn(move|| {
                            println!("Validation successfull: starting timeout");
                            thread::sleep(Duration::from_secs(constants::PASS_DELAY));
                            let mut validation_a = validated_clone.lock().unwrap();
                            *validation_a = false;
                            let mut valid_password_a = valid_password_clone.lock().unwrap();
                            *valid_password_a = String::from("");
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

fn client(mut stream: TcpStream, mut validate: &mut bool, mut valid_password: &mut String) -> bool {
    let mut data = [0 as u8; 50]; // using 50 byte buffer
    match stream.read(&mut data) {
        Ok(size) => {
            let before_validate = *validate;
            let flag = &data[0];
            let content = from_utf8(&data[1..size]).unwrap();
            let mut args: Vec<&str> = content.split("#@#").collect();
            println!("content from client: {:?}", content);
            let response: String = match Event::from(flag) {
                Event::New => handle_new(&mut validate, &args, &valid_password),
                Event::Get => handle_get(&mut validate, &mut args, &valid_password),
                Event::Validate => handle_validate(&content, &mut validate, &mut valid_password),
                Event::ChangeMP => handle_change_mp(&content, &mut validate),
                Event::Init => handle_init(&content),
                Event::List => handle_list()
            };
            let response = response.into_bytes();
            let mut chained_response = vec![];
            chained_response.push(*validate as u8);
            for i in response.iter(){
                chained_response.push(*i as u8);

            }
            let boxed: Box<[u8]> = chained_response.into_boxed_slice();
            stream.write(&boxed).unwrap();
            if before_validate == false && *validate == true {
                true
            } else {
                false
            }
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    }
}

// {{{ utils
fn set_password(pass: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.input_str(pass);
    let hash = hasher.result_str();
    let root = env::var("XDG_CONFIG_HOME").unwrap();
    let path = Path::new(root.as_str());
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path.join(constants::PASSWORD_HASH_HOLDER))
        .unwrap();

    if let Err(e) = writeln!(file, "{}", &hash) {
        panic!("Couldn't write to hash holder file: {}", e);
    }
    "Password set to given value".to_string()
}

fn is_password_valid(pass: &str) -> Result<(), Reason> {
    let mut hasher = Sha256::new();
    hasher.input_str(pass);
    let hash = hasher.result_str();
    let root = env::var("XDG_CONFIG_HOME").unwrap();
    let path = Path::new(root.as_str());
    let old_hash = fs::read_to_string(path.join(constants::PASSWORD_HASH_HOLDER))
        .expect("Something went wrong reading the hash file");
    if old_hash.trim().is_empty() {
        return Err(Reason::PasswordEmpty);
    }
    if hash != old_hash.trim() {
        return Err(Reason::PasswordInvalid);
    }
    Ok(())
}

fn cut_into_pieces(data: &str) -> Vec<u8> {
    data.split("#@#").map(|num_string| num_string.parse::<u8>().unwrap()).collect()
}

fn key_used(key: &str) -> bool {
    let storage_map = parse_storage();
    if storage_map.contains_key(key) {
        return true;
    }
    false
}

fn parse_storage() -> HashMap<String, String> {
    let root = env::var("XDG_CONFIG_HOME").unwrap();
    let path = Path::new(root.as_str());
    let file = File::open(path.join(constants::STORAGE)).unwrap();
    let reader = BufReader::new(file);

    let mut result: HashMap<String, String> = HashMap::new();
    for line in reader.lines() {
        let line_string = line.unwrap();
        let values: Vec<&str> = line_string.split(" ||| ").collect();
        result.insert(values[0].to_string(), values[1].to_string());
    }
    result
}

fn has_login_flag(args: &mut Vec<&str>) -> bool {
    let result = args.contains(&"-l");
    args.retain(|&x| x != "-l");
    result
}

// }}} utils

// {{{ handles
fn handle_list() -> String {
    let map = parse_storage();
    let mut response = String::new();
    for key in map.keys() {
        response.push_str(key);
        response.push_str("\n");
    }
    response.trim_end().to_string()

}

fn handle_init(pass: &str) -> String {
    let validation_res = is_password_valid(pass);
    match validation_res {
        Ok(()) => {
            "Given password already matches current one, nothing changed".to_string()
        },
        Err(reason) => {
            match reason {
                Reason::PasswordInvalid => "Password already set, to change it, use change command".to_string(),
                Reason::PasswordEmpty => set_password(pass)
            }
        }
    }
}

fn handle_change_mp(pass: &str, validated: &mut bool) -> String {
    if *validated {
        return set_password(pass)
    } else {
        "Not validated".to_string()
    }
}

fn handle_validate(pass: &str, validated: &mut bool, valid_password: &mut String) -> String {
    let validation_res = is_password_valid(pass);
    match validation_res {
        Ok(()) => {
            *validated = true;
            *valid_password = pass.to_string();
            "Successfully validated".to_string()
        },
        Err(reason) => {
            *validated = false;
            *valid_password = String::from("");
            reason.to_string()
        }
    }
}

fn handle_new(validated: &mut bool, args: &Vec<&str>, valid_password: &str) -> String {
    if *validated {
        if args.len() < 2 {
            return "Exception: Not enough arguments given".to_string();
        }
        if key_used(&args[0]) {
            return "Exception: Key already used".to_string();
        }
        let record: Record;
        if args.len() < 3 {
            let message = args[1].to_string();
            let encrypted_data = encode(&message, &valid_password);
            record = Record {
                key: args[0].to_string(),
                data: encrypted_data
            }
        } else {
            let message = format!("{}#@#{}", args[1], args[2]);
            let encrypted_data = encode(&message, &valid_password);
            record = Record {
                key: args[0].to_string(),
                data: encrypted_data
            }
        }
        record.save();
        return "Record saved!".to_string();
    } else {
        return "Exception: not validated".to_string();
    }
}

fn handle_get(validated: &mut bool, mut args: &mut Vec<&str>, valid_password: &str) -> String {
    let return_login =  has_login_flag(&mut args);
    if *validated {
        if ! key_used(&args[0]) {
            return "Exception: Key does not exist".to_string();
        }
        let map: HashMap<String, String> = parse_storage();
        let encrypted_data = map.get(&args[0].to_string()).unwrap();
        let decrypted_stuff = decode(&encrypted_data, &valid_password);
        let pieces: Vec<&str> = decrypted_stuff.split("#@#").collect();
        if return_login {
            if pieces.len() < 2 {
                return "Exception: Record does not contain login data".to_string();
            } else {
                return pieces[0].to_string();
            }
        } else {
            if pieces.len() > 1 {
                return pieces[1].to_string();
            } else {
                return pieces[0].to_string();
            }
        }
    } else {
        return "Exception: not validated".to_string();
    }
}

// }}} handles

// {{{ encrypt/decrypt

fn decode(data: &String, key: &str) -> String {
    let data_array: Vec<u8> = cut_into_pieces(&data);
    let cipher = Cipher::aes_128_cbc();
    let key_bytes = key.as_bytes();
    let mut key_array: [u8; 16] = [0; 16];
    make_sure_len_16(key_bytes, &mut key_array);
    let ciphertext = decrypt(
        cipher,
        &key_array,
        Some(constants::IV),
        &data_array)
        .unwrap();
    let decryted_string = from_utf8(&ciphertext[..]).unwrap();
    decryted_string.to_string()
}
fn encode(data: &String, key: &str) -> String {
    let cipher = Cipher::aes_128_cbc();
    let key_bytes = key.as_bytes();
    let mut key_array: [u8; 16] = [0; 16];
    make_sure_len_16(key_bytes, &mut key_array);
    let data = data.as_bytes();
    let ciphertext = encrypt(
        cipher,
        &key_array,
        Some(constants::IV),
        data)
        .unwrap();
    let mut result = String::new();
    for (i, num) in ciphertext.iter().enumerate() {
        result.push_str(&num.to_string());
        if i < ciphertext.len() - 1 {
            result.push_str("#@#");
        }
    }
    result
}
fn make_sure_len_16(key_bytes: &[u8], result: &mut [u8]) {
    for i in 0..16 {
        result[i] = key_bytes[i%key_bytes.len()];
    }
}
// }}}
