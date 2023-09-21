use brewdrivers::{model::RTU, state::BinaryState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("trace"));

    let mut rtu = RTU::generate().unwrap();

    let some_device = rtu.devices.get_mut(1).unwrap();
    some_device.state.relay_state = Some(BinaryState::On);

    some_device.enact().await?;

    Ok(())
}
