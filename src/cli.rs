//! This is the command line interface (CLI) for `brewdrivers`.
//! It provides a way to a) run the brewdrivers server and
//! b) interact with the hardware directly from the
//! command line (normally for debugging and testing).
//!
//! To run it, just run the `brewdrivers` binary provided in the Github releases,
//! or you can run it from Rust through the [`cli::run`](fn.run.html) function
//! # Commands
//! ## Server
//! ```
//! Coming soon...
//! ```
//! ## Relays
//! From the CLI you can:
//! 1. Set a relay
//! 2. Get a relay status
//! 3. Get all relay statuses
//! 4. Set controller number
//!
//! ```
//! // Set a relay
//! // Provide controller number, relay number, and new state
//! // This turns relay 4 on controller #2 on
//! $ brewdrivers relay 2 4 1
//! // And back off again
//! $ brewdrivers relay 2 4 0
//!
//! // Get a relay status
//! // Same as above, but don't provide a state
//! $ brewdrivers relay 2 4
//!
//! // Get all relay statuses
//! $ brewdrivers relay 2 all
//! Relay 0: Off
//! Relay 1: On
//! ...
//!
//! // Set controller number
//! Coming soon...
//! ```
//!

use std::process;

use crate::relays::Str1xx;
use crate::relays::State;

use clap::{Arg, App, SubCommand, ArgMatches};

fn matches() -> ArgMatches<'static> {
    return App::new("Brewdrivers")
        .version(env!("CARGO_PKG_VERSION"))
        .author("llamicron <llamicron@gmail.com>")
        .about("Hardware drivers")
        .subcommand(SubCommand::with_name("relay")
            .about("Controls an STR116 or STR008")
            .arg(Arg::with_name("controller_num")
                .help("Board controller number")
                .validator(is_int)
                .required(true)
                .index(1))
            .arg(Arg::with_name("relay_num")
                .help("Relay to change")
                .validator(is_int_or_all)
                .required(true)
                .index(2))
            .arg(Arg::with_name("state")
                .help("Relay state: 1 or 0")
                .validator(is_int)
                .required(false)
                .index(3)))

    .get_matches();
}

/// Runs the CLi
pub fn run() {
    let matches = matches();

    if let Some(matches) = matches.subcommand_matches("relay") {
        handle_relay_matches(&matches);
    }

}

fn handle_relay_matches(matches: &ArgMatches) {
        let cn = matches.value_of("controller_num").unwrap().parse::<u8>().unwrap();
        let mut str116 = Str1xx::new(cn);

        let rn_matches = matches.value_of("relay_num").unwrap();

        if rn_matches == "all" {
            str116.list_all_relays();
            process::exit(0);
        }

        let rn = rn_matches.parse::<u8>().unwrap();

        if let Some(state) = matches.value_of("state") {
            match state.parse::<u8>().unwrap() {
                1 => {
                    println!("*click*\nTurning relay {} on controller {} on", rn, cn);
                    str116.set_relay(rn, State::On)
                },
                0 => {
                    println!("*click*\nTurning relay {} on controller {} off", rn, cn);
                    str116.set_relay(rn, State::Off)
                },
                _ => {}
            };
        } else {
            println!("Controller {} relay {} is {:?}", cn, rn, str116.get_relay(rn));
        }
}

fn is_int(arg: String) -> Result<(), String> {
    match arg.parse::<u8>() {
        Ok(_) => return Ok(()),
        Err(_) => return Err("needs to be an integer".to_string())
    }
}

fn is_int_or_all(arg: String) -> Result<(), String> {
    match arg.parse::<u8>() {
        Ok(_) => return Ok(()),
        Err(_) => {
            if arg == "all".to_string() { return Ok(()); }
        }
    }
    Err("needs to be an int or 'all'".to_string())
}
