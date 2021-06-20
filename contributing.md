# Contributing
This is the central repository for device drivers that the Navasota Brewing Company uses. We support open source software, and welcome any additions to our library of device drivers.

## Code Layout
Within the `src/` directory, there should be a directory for each "category" of device. I use the term "category" loosely. For example, there are currently 2 directories in `src/`:

```
├── omega
│   ├── cn7500.rs
│   └── mod.rs
└── relays
    ├── mod.rs
    ├── str1
    │   ├── bytestring.rs
    │   ├── mod.rs
    │   └── str1.rs
    └── waveshare.rs
```

The `omega/` directory contains drivers for OMEGA Engineering devices; we use the CN7500, so there is a `cn7500.rs` module. `omega/mod.rs` contains structs and functions that may be common to all OMEGA products. If we decide to support another OMEGA product, we would add another module in `omega/`, and reuse whatever components from `mod.rs` or `cn7500.rs` that we could.

For `relays/`, the `str1/` directory contains things to run an STR1 board, while the `waveshare.rs` module fits in one file and doesn't need a directory. `str1` and `waveshare` are siblings.

Devices could be grouped by family, manufacturer, function, or name. The directory structure should just follow common sense. Things should go where it seems they should go.

## Documentation and Testing
All added code should be well documented and tested. I may refuse or delay a driver's inclusion here until it's tested and documented. 
