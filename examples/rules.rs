use log::*;

use brewdrivers::model::rules::RuleSet;
use brewdrivers::model::RTU;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("trace"));

    let rules = RuleSet::get_all()?;

    for rule in &rules.0 {
        info!("Found rule {}", rule.name);
    }

    let mut rtu = RTU::generate(None)?;

    for rule in &rules.0 {
        rule.apply(rtu.devices.iter_mut().collect()).await?;
    }

    Ok(())
}
