# Brewdrivers Guides

**Note:** This is not the documentation for `brewdrivers`. These are guides for brewery design and hardware. For the code documentation, see [here](https://docs.rs/brewdrivers/).


## Table of Contents
 * [Relay Boards](relays.md)


## Overview
This project provides low level drivers for the hardware that the [Navasota Brewing Cooperative](https://navasotabrewing.com) uses on it's brewing rig. It serves as a back end to [brewkit](https://github.com/NavasotaBrewing/brewkit), a project that makes it easy to manage and control the brewing hardware. It's basically a small SCADA system. `Brewkit` is the web front-end with pretty graphics and nice things, `brewdrivers` is where all the low level drivers live.

This project creates a web API that brewkit uses, but can also be used directly. The command line interface provides a way to interact with the hardware directly. We use it for debugging in a pinch, or a quick fix. See the CLI [documentation](https://docs.rs/brewdrivers/) (click the `cli` module) for instructions. Another method is through Rust itself. This project is published as a [crate on crates.io](https://docs.rs/crate/brewdrivers). You can see the [documentation](https://docs.rs/brewdrivers/) for instructions specific to each piece of hardware.

For every class of hardware we use, there will be a Rust module (a subdir in the `src/` directory). For example, everything related to relays and relay boards are in the `relays` module. Within each module will be specific implementations of that class of hardware. So in the `relays` module, there is `str1.rs`, a driver for `STR1` relay boards.


## Links
 * [Brewdrivers documentation](https://docs.rs/brewdrivers/)
