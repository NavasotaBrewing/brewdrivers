# Brewdrivers
These are low level drivers for various controllers used in brewing. They will eventually be implemented by [brewkit](https://github.com/NavasotaBrewing/brewkit). Currently, brewkit is using python drivers, but we're moving to port them to rust, which is what this project is.

## Controllers
There are currently two controllers we use. One is an STR116 relay board. This can control lots of devices, mostly valves and pumps. The other is an OmegaCN7500 PID. It regulates temperatures and controls a RIMS heater.

### `Str116`
```rust

use str116::{Str116, State::{On, Off}};

fn main() {
    // Use the address programmed into the board beforehand. 2, in this case.
    let mut controller = Str116::new(2);

    // Set a relay on or off
    controller.set_relay(1, On);
    controller.set_relay(1, Off);
}
```

### `Omega`
Not yet implemented...
