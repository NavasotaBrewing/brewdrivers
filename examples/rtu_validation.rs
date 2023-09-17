use brewdrivers::model::RTU;
use env_logger;

fn main() {
    // This will generate an RTU from the default location (look in defaults.rs in brewdrivers)
    // and print the logging output from it. Can be useful to test if the config file is valid.
    //
    // I think there's also an argument in the CLI package that will validate the rtu config
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("trace"));

    if let Err(e) = RTU::generate(None) {
        log::error!("{e}");
    }
}
