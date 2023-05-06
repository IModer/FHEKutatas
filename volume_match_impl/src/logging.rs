use std::fs::OpenOptions;
use std::io::Write;
use std::string;
use std::time::Duration;

pub fn log(name : &str, time : Duration)
{
    let mut file = OpenOptions::new()
    .create(true)
    .append(true)
    .open("runtime.log")
    .unwrap();
    
    if let Err(e) = writeln!(file, "LOG: {} {:.4?} ", name, time) {
    eprintln!("Couldn't write to file: {}", e);
    }
}
