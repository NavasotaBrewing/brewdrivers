use std::env;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("Enter one of the following: ");
        println!("omega:   Runs the Omega CLI");
        println!("relay:   Runs the relay CLI");
        println!("master:  Starts the master API");
        println!("rtu:     Starts the RTU API");
        exit(0);
    }

    match args[1].as_str() {
        "omega" => brewdrivers::cli::omega(),
        "relay" => brewdrivers::cli::relay(),
        "master" => brewdrivers::master::api::run(),
        "rtu" => brewdrivers::RTU::api::run(),
        _ => {}
    }
}
