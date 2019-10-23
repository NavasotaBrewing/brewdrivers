use std::fmt;

// CN7500, STR1
use crate::RTU::*;

use shrust::{Shell, ShellIO};
use std::io::prelude::*;

// This is used only for the cli, don't get it confused with Configuration
#[derive(Debug)]
struct ControllerConfig {
    addr: u8,
    baudrate: u32,
    port: String
}

impl fmt::Display for ControllerConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "-- Controller Config --")?;
        writeln!(f, "Port:      {}", self.port)?;
        writeln!(f, "Address:   {}", self.addr)?;
        write!(f,   "Baudrate:  {}", self.baudrate)
    }
}


fn controller_shell() -> Shell<ControllerConfig> {
    let config = ControllerConfig {
        addr: 0x16,
        baudrate: 19200,
        port: String::from("/dev/ttyAMA0"),
    };

    let mut shell = Shell::new(config);

    shell.new_command_noargs("version", "View the current version", |io, _| {
        writeln!(io, "Brewdrivers v{}", env!("CARGO_PKG_VERSION"))?;
        Ok(())
    });

    // Omega
    shell.new_command("with_addr", "Configure the address of the controller (decimal)", 1, |io, config, addrs| {
        match addrs[0].parse::<u8>() {
            Ok(addr) => {
                config.addr = addr;
                writeln!(io, "Now using address {}", addr)?;
            },
            Err(e) => {
                writeln!(io, "{}", e)?;
            }
        };
        Ok(())
    });

    shell.new_command("with_port", "Configure the port of the controller", 1, |io, config, ports| {
        match ports[0].parse::<String>() {
            Ok(port) => {
                writeln!(io, "Now using port {}", port)?;
                config.port = port;
            },
            Err(e) => {
                writeln!(io, "{}", e)?;
            }
        };
        Ok(())
    });

    shell.new_command("with_baud", "Configure the baudrate of the controller", 1, |io, config, bauds| {
        match bauds[0].parse::<u32>() {
            Ok(baud) => {
                config.baudrate = baud;
                writeln!(io, "Now using baudrate {}", baud)?;
            },
            Err(e) => {
                writeln!(io, "{}", e)?;
            }
        };
        Ok(())
    });

    shell.new_command_noargs("config", "View the current controller configuration", |io, config| {
        writeln!(io, "{}", config)?;
        Ok(())
    });

    shell
}

fn newCN7500(config: &ControllerConfig) -> CN7500 {
    CN7500::new(config.addr, &config.port, config.baudrate)
}

fn newSTR1(config: &ControllerConfig) -> STR1 {
    STR1::with_address(config.addr)
}

pub fn omega() {
    println!("Entering Omega CLI");

    let mut shell = controller_shell();

    shell.new_command("set_sv", "Set the setpoint value", 1, |io, config, new_temp| {
        let cn = newCN7500(&config);

        match new_temp[0].parse::<f64>() {
            Ok(temperature) => {
                if temperature > 752.0 || temperature < 0.1 {
                    writeln!(io, "Temperature out of range (0.1-752)")?;
                } else {
                    cn.set_sv(temperature);
                    writeln!(io, "Setting to {}", temperature)?;
                }
            },
            Err(_) => writeln!(io, "Not a number")?
        };

        Ok(())
    });

    shell.new_command_noargs("pv", "Get the process value", |io, config| {
        let cn = newCN7500(&config);
        writeln!(io, "Process: {}", cn.get_pv())?;
        Ok(())
    });

    shell.new_command_noargs("sv", "Get the setpoint value", |io, config| {
        let cn = newCN7500(&config);
        writeln!(io, "Setpoint: {}", cn.get_sv())?;
        Ok(())
    });

    shell.new_command_noargs("run", "Run the relay", |io, config| {
        let cn = newCN7500(&config);
        cn.run();
        writeln!(io, "running...")?;
        Ok(())
    });

    shell.new_command_noargs("stop", "Stop the relay", |io, config| {
        let cn = newCN7500(&config);
        cn.stop();
        writeln!(io, "stopped.")?;
        Ok(())
    });

    shell.new_command_noargs("is_running", "Checks if the relay is running", |io, config| {
        let cn = newCN7500(&config);
        writeln!(io, "{}", cn.is_running())?;
        Ok(())
    });

    shell.run_loop(&mut ShellIO::default());
}

pub fn relay() {
    println!("Entering relay CLI");

    let mut shell = controller_shell();

    shell.new_command("get_relay", "Prints the status of a relay", 1, |io, config, relay_nums| {
        let mut board = newSTR1(&config);

        for num in relay_nums {
            match num.parse::<u8>() {
                Ok(relay) => writeln!(io, "Relay {} is {}", relay, board.get_relay(relay)).unwrap(),
                Err(e) => writeln!(io, "Not a valid relay number: {}", e).unwrap()
            }
        }

        Ok(())
    });

    shell.new_command("set_relay", "Sets a relay on or off", 2, |io, config, args| {
        let mut board = newSTR1(&config);
        if args.len() > 2 {
            writeln!(io, "Error: Too many args").unwrap();
        }

        let mut state: State = State::Off;

        if args[1] == "1" {
            state = State::On;
        }

        match args[0].parse::<u8>() {
            Ok(relay) => {
                board.set_relay(relay, state);
                writeln!(io, "Relay {} is {}", relay, board.get_relay(relay)).unwrap();
            },
            Err(e) => writeln!(io, "Not a valid relay number: {}", e).unwrap(),
        }

        Ok(())
    });

    shell.new_command("set_cn", "Change the controller number of the baord", 1, |io, config, args| {
        let mut board = newSTR1(&config);

        match args[0].parse::<u8>() {
            Ok(new_cn) => {
                board.set_controller_num(new_cn);
                writeln!(io, "Controller number set to {}", new_cn).unwrap();
            },
            Err(e) => writeln!(io, "Invalid controller number: {}", e).unwrap(),
        }

        Ok(())
    });

    shell.run_loop(&mut ShellIO::default());
}
