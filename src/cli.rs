use std::process;

use crate::relays::Str1xx;
use crate::relays::State;

use clap::{Arg, App, SubCommand, ArgMatches};

pub fn matches() -> ArgMatches<'static> {
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

pub fn run() {
    let matches = matches();

    if let Some(matches) = matches.subcommand_matches("relay") {
        handle_relay_matches(&matches);
    }

}

fn handle_relay_matches(matches: &ArgMatches) {
        let cn = matches.value_of("controller_num").unwrap().parse::<u8>().unwrap();
        let mut str116 = Str1xx::new(cn);

        let rn_match = matches.value_of("relay_num").unwrap();
        let rn = match rn_match.parse::<u8>() {
            Ok(x) => x,
            Err(_) => {
                if rn_match == "all" {
                    println!("Controller {}", cn);
                    for i in 0..16 { println!("Relay {}: {:>6?}", i, str116.get_relay(i)); }
                }
                process::exit(0);
            }
        };



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
