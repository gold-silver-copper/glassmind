use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        match glassmind_english_rust::implement::request(&line) {
            Some(Ok(action)) => println!("{}", action.target),
            Some(Err(_)) => println!("REFUSED"),
            None => println!("UNRECOGNIZED"),
        }
    }
}
