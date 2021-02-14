use std::env;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("Enter one of the following: ");
        println!("omega:   Runs the Omega CLI");
        println!("relay:   Runs the relay CLI");
        exit(0);
    }

    match args[1].as_str() {
        "omega" => brewdrivers::cli::omega(),
        "relay" => brewdrivers::cli::relay(),
        _ => {}
    }
}
