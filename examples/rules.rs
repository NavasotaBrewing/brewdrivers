use log::*;

use brewdrivers::model::rules::RuleSet;
use brewdrivers::model::Device;
use brewdrivers::model::RTU;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("trace"));

    let rules = RuleSet::get_all()?;

    for rule in &rules.0 {
        info!("Found rule {}", rule.name);
    }

    let rtu = RTU::generate(None)?;

    // rtu.update().await?;
    let mut devices: Vec<Device> = rtu
        .devices
        .into_iter()
        .filter(|dev| dev.id == "omega1" || dev.id == "relay0")
        .collect();

    for device in devices.iter_mut() {
        device.update().await?;
    }

    for rule in &rules.0 {
        rule.apply(devices.iter_mut().collect()).await?;
    }

    for device in devices.iter_mut() {
        device.enact().await?;
    }
    // rtu.enact().await?;

    Ok(())
}
