# Brewdrivers
This repository is one of a set of repositories in the Brewery Control System project of Navasota Brewing Company. It contains low level drivers for devices we use in the brewing process.

Unless you're here looking to do some direct interaction with devices and RTUs, this probably isn't the best place to start. See the [organization's readme](https://github.com/NavasotaBrewing/readme) for more information.

# Usage
This package is mostly meant as a library for our other packages, but it also contains a CLI interface to interact directly with the devices.

This package can be installed with `cargo`
```
$ cargo install brewdrivers
$ brewdrivers
Enter one of the following: 
omega:   Runs the Omega CLI
relay:   Runs the relay CLI
...
```

There should be a CLI interface for each type of device we support. Enter each one with the command line argument specified.

## Relay CLI
Here, I enter the relay CLI and configure my address. I have my STR108 board set to address 1. The default here is 22. You can set the baudrate and port in the same way. type `help` for specific help.
```
$ brewdrivers relay
Entering relay CLI
> config
-- Controller Config --
Port:      /dev/ttyAMA0
Address:   22
Baudrate:  19200
> with_addr 1
Now using address 1
> set_relay 1 1
Relay 1 is on
> set_relay 1 0
Relay 1 is off
```

## Omega CLI
Just like with the relays, you should be sure the config data matches how you have your hardware configured. The defaults for the config data are how my CN7500 is set up, so I won't change anything.

```
$ brewdrivers omega
Entering omega CLI
> config
-- Controller Config --
Port:      /dev/ttyAMA0
Address:   22
Baudrate:  19200
> pv
114.3
> sv
175.0
> set_sv 145.5
145.5
> sv
145.5
> run
running...
> stop
stopped.
> help
...
```

For information about the hardware we support, see the [organization's readme](https://github.com/NavasotaBrewing/readme).