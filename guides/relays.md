# Relays

## What are they
Relay boards contain relays, which can be on or off. Physical devices like valves and pumps
can be connected to relays in order to be toggled on and off. Any simple device on our brew rig that can be toggled, like a pump or a valve, will use a relay.

There are many different types of relay boards. They have been used for decades in all sorts of applications. We use the [SmartHardware STR1](https://www.smarthardware.eu/index.php) line of relay boards. They are a reliable board at a reasonable price. SmartHardware also provides a [software guide](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf) on how to communicate with the board over RS-485. We provide a driver for the STR1 line of boards, contained in this Github repo. All STR1 boards should behave the same.

## Hardware
Coming soon...

## Software
This repository contains a driver for the STR1 line of relay boards. After installing, there are two ways to use it:

1. The [command line interface](https://docs.rs/brewdrivers/0.2.2/brewdrivers/cli/index.html#relays)
2. Through the Rust API, see the [relays module](https://docs.rs/brewdrivers/0.2.2/brewdrivers/relays/index.html)

**Note:** if you just connected a board, you'll probably want to set the controller number. You can do this through the CLI or through the Rust API. See the documentation linked above.
