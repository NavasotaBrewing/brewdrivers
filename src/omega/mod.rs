pub mod cn7500;
pub use cn7500::CN7500;


#[derive(Debug)]
pub enum Degree {
    Fahrenheit,
    Celsius
}
