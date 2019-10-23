use std::fmt;

use crate::RTU::CN7500;
use crate::RTU::STR1;

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

pub fn omega() {
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
    let mut shell = controller_shell();

    shell.run_loop(&mut ShellIO::default());
}
