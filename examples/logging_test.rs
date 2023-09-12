use brewdrivers::{logging_utils::*, model::Device};

fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("trace"));
    let device: Device = serde_yaml::from_str(
        r#"
        id: pump
        name: Pump
        conn:
            port: /dev/ttyUSB0
            baudrate: 9600
            timeout: 100
            controller: STR1
            controller_addr: 254
            addr: 0
    "#,
    )
    .unwrap();

    // You can just give it a device
    device_trace!(device);
    device_debug!(device);
    device_info!(device);
    device_warn!(device);
    device_error!(device);

    // Or provide an additional message
    device_trace!(device, "Device said hello!");
    device_debug!(device, "Device said hello!");
    device_info!(device, "Device said hello!");
    device_warn!(device, "Device said hello!");
    device_error!(device, "Device said hello!");
}

