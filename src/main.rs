use brewdrivers::cli;
use hex;

fn main() {
    // cli::parse_args();
    assert_eq!(hex::decode("fe").unwrap()[0], 254);
}
