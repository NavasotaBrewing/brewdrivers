use std::process;

use crate::RTU::relays::{STR1, State, Board};
use crate::master::socket as master_socket;
use crate::RTU::socket as rtu_socket;

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
        .subcommand(SubCommand::with_name("socket")
            .about("Runs the Master WebSocket")
            .arg(Arg::with_name("position")
                .help("'master' or 'rtu'")
                .validator(validators::is_master_or_rtu)
                .required(true)
                .index(1)))

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

    if let Some(matches) = matches.subcommand_matches("socket") {
        if matches.value_of("position").unwrap() == "master" {
            master_socket::run();
        } else {
            rtu_socket::run();
        }
    }

}

fn handle_set_cn_matches(matches: &ArgMatches) {

    let ccn = matches.value_of("current_cn").unwrap().parse::<u8>().unwrap();
    let ncn = matches.value_of("new_cn").unwrap().parse::<u8>().unwrap();

    let mut board = STR1::with_address(ccn);
    board.set_controller_num(ncn);
    println!("Controller {} -> Controller {}", ccn, ncn);
}

fn handle_relay_matches(matches: &ArgMatches) {
    let cn = matches.value_of("controller_num").unwrap().parse::<u8>().unwrap();
    let mut str116 = STR1::with_address(cn);

    let rn_matches = matches.value_of("relay_num").unwrap();

    if rn_matches == "all" {
        str116.list_all_relays();
        process::exit(0);
    }

    let rn = rn_matches.parse::<u8>().unwrap();

    if let Some(state) = matches.value_of("state") {
        let state_bool = state.parse::<u8>().unwrap() != 0;
        let state = State::from(state_bool);
        str116.set_relay(rn, state);
    }
    println!("Controller {} relay {} is {}", cn, rn, str116.get_relay(rn));
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

    pub fn is_master_or_rtu(arg: String) -> Result<(), String> {
        if arg.as_str() == "master" || arg.as_str() == "rtu" {
            return Ok(())
        }
        Err("Acceptable values are 'master' or 'rtu'".to_string())
    }
}
