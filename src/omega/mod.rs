//! Drivers for OMEGA Engineering instruments.
pub mod cn7500;
pub use cn7500::CN7500;


/// Represents `F` or `C` units for the CN7500.
#[derive(Debug)]
pub enum Degree {
    Fahrenheit,
    Celsius
}
