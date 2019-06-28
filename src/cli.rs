//! Command line interaction for all driver modules
//!
//! This is the command line interface (CLI) for `brewdrivers`.
//! It provides a way to interact with the hardware directly (normally for debugging and testing).
//!
//! To run it, just run the `brewdrivers` binary provided in the Github releases, or compile the latest version of `brewdrivers`.
//!
//!
//! # Commands
//! You can always run `brewdrivers help` or `brewdrivers <subcommand> help`
//! to get a list of parameters, extra information about a command, version numbers, etc.
//!
//! ## Relays
//!
//! ```text
//! // Set a relay
//! // Provide controller number, relay number, and new state
//! // This turns relay 4 on controller 2 on
//! $ brewdrivers relay 2 4 1
//! // And back off again
//! $ brewdrivers relay 2 4 0
//!
//! // Get a relay status
//! // Same as above, but don't provide a state
//! $ brewdrivers relay 2 4
//!
//! // Get all relay statuses on controller 2
//! $ brewdrivers relay 2 all
//! Relay 0: Off
//! Relay 1: On
//! ...
//!
//! // Set controller number
//! $ brewdrivers set_cn 2 3
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
                .validator(validators::is_int)
                .required(true)
                .index(1))
            .arg(Arg::with_name("relay_num")
                .help("Relay to change")
                .validator(validators::is_int_or_all)
                .required(true)
                .index(2))
            .arg(Arg::with_name("state")
                .help("Relay state: 1 or 0")
                .validator(validators::is_int)
                .required(false)
                .index(3)))
        .subcommand(SubCommand::with_name("set_cn")
            .about("Programs the controller number of a board")
            .arg(Arg::with_name("current_cn")
                .help("Current controller number")
                .validator(validators::is_int)
                .required(true)
                .index(1))
            .arg(Arg::with_name("new_cn")
                .help("New controller number to set (0-255)")
                .validator(validators::is_int)
                .required(true)
                .index(2)))

    .get_matches();
}

pub fn parse_args() {
    let matches = matches();

    if let Some(matches) = matches.subcommand_matches("relay") {
        handle_relay_matches(&matches);
    }

    if let Some(matches) = matches.subcommand_matches("set_cn") {
        handle_set_cn_matches(matches);
    }
}

fn handle_set_cn_matches(matches: &ArgMatches) {

    let ccn = matches.value_of("current_cn").unwrap().parse::<u8>().unwrap();
    let ncn = matches.value_of("new_cn").unwrap().parse::<u8>().unwrap();

    let mut board = Str1xx::new(ccn);
    board.set_controller_num(ncn);
    println!("Controller {} -> Controller {}", ccn, ncn);
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
                println!("*click*\nController {}, relay {} on", cn, rn);
                str116.set_relay(rn, State::On)
            },
            0 => {
                println!("*click*\nController {}, relay {} off", cn, rn);
                str116.set_relay(rn, State::Off)
            },
            _ => {}
        };
    } else {
        println!("Controller {} relay {} is {:?}", cn, rn, str116.get_relay(rn));
    }
}


mod validators {
    pub fn is_int(arg: String) -> Result<(), String> {
        match arg.parse::<u8>() {
            Ok(_) => return Ok(()),
            Err(_) => return Err("needs to be an integer".to_string())
        }
    }

    pub fn is_int_or_all(arg: String) -> Result<(), String> {
        match arg.parse::<u8>() {
            Ok(_) => return Ok(()),
            Err(_) => {
                if arg == "all".to_string() { return Ok(()); }
            }
        }
        Err("needs to be an int or 'all'".to_string())
    }
}
