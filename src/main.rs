extern crate synacor;

use std::env;
use self::synacor::*;

fn main() {
    match env::args().nth(1) {
        None => println!("Please provide an input path."),
        Some(path) => {
            let mut machine = Machine::new();
            let read = machine.load(&path).unwrap_or(0);
            println!("Read {} bytes, executing.", read);
            println!("=========================");
            machine.run();
        }
    }
}
