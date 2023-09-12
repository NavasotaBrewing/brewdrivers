use brewdrivers::{logging_utils::*, model::Device};

fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("trace"));
    let device: Device = serde_yaml::from_str(r#"
        id: pump
        name: Pump
        conn:
            port: /dev/ttyUSB0
            baudrate: 9600
            timeout: 100
            controller: STR1
            controller_addr: 254
            addr: 0
    "#).unwrap();
    trace!(device);
    trace!(device, "Device said hello!");
    debug!(device, "Device said hello!");
    info!(device, "Device said hello!");
    warn!(device, "Device said hello!");
    error!(device, "Device said hello!");
}