use std::fs::OpenOptions;
use std::io::Write;
use std::string;

fn log(name : String , time : u64)
{
    let mut file = OpenOptions::new()
    .create(true)
    .append(true)
    .open("runtime.log")
    .unwrap();
    
    if let Err(e) = writeln!(file, "LOG: {} {} ", name, time) {
    eprintln!("Couldn't write to file: {}", e);
    }
}
