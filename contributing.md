# Contributing
This is the central repository for device drivers that the Navasota Brewing Company uses. We support open source software, and welcome any additions to our library of device drivers.

## Code Layout
Within the `src/` directory, there should be a directory for each "category" of device. I use the term "category" loosely. For example, there are current 2 directories in `src/`:

```
├── omega
│   ├── cn7500.rs
│   └── mod.rs
└── relays
    ├── bytestring.rs
    ├── mod.rs
    └── str1.rs
```

The `omega/` directory contains drivers for OMEGA Engineering devices; we use the CN7500, so there is a `cn7500.rs` module. `omega/mod.rs` contains structs and functions that may be common to all OMEGA products. If we decide to support another OMEGA product, we would add another module in `omega/`, and reuse whatever components from `mod.rs` or `cn7500.rs` that we could.

`relays/` is a little different. Once again, `relays/mod.rs` and `bytestring.rs` contain things that may be common to all relay boards. The `str1.rs` module is a driver for the `STR116` and `STR108` relay boards specifically. We may add support for more relay boards, perhaps of a completely different type, in the `relays/` directory.

Devices could be grouped by family, manufacturer, function, or name. The directory structure should just follow common sense. Things should go where it seems they should go.

## Documentation and Testing
All added code should be well documented and tested. I may refuse or delay a driver's inclusion here until it's tested and documented. 
