/// This module handles validation for all the model structs.
///
/// In a nutshell, it will take a generated RTU, Rule Set, Condition collection, etc, and make sure
/// that they are actually compatible with each other and valid. For example, if you define a
/// device with a port that doesn't exist, or you define a rule over a device that doesn't exist,
/// these validators will catch this and report the errors.
///
/// Note that the model structs will fail to properly deserialize if there is a YAML syntax error.
/// These validators do not check YAML syntax error, instead this happens earlier through serde.
use log::*;

use super::RTU;
use crate::error::Error;

pub mod rtu_validators;

pub fn validate_all(rtu: &mut RTU) -> Result<(), Vec<Error>> {
    let mut all_errors = Vec::new();

    match rtu_validators::all(rtu) {
        Ok(_) => info!("RTU validation passed"),
        Err(errors) => {
            error!("RTU validation failed with the following errors:");
            for error in errors {
                error!("{error}");
                all_errors.push(error);
            }
        }
    }

    if all_errors.len() == 0 {
        info!("All validation passed with no errors");
        return Ok(());
    }
    Err(all_errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_validation() {
        let mut rtu = RTU::generate().unwrap();

        assert!(validate_all(&mut rtu).is_ok());
    }
}
