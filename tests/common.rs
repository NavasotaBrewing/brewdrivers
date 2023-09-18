use brewdrivers::controllers::Controller;
use brewdrivers::model::{Device, RTU};

/// Deserializes the configuration and finds the first device in it with the
/// right controller type, if any
/// TODO: this is duplicated by crate::tests::test_device_from_type. Replace one of them, probably
/// this one
pub fn get_device_from_configuration(controller_type: Controller) -> Option<Device> {
    let rtu = RTU::generate().unwrap();
    rtu.devices
        .iter()
        .find(|dev| *dev.conn.controller() == controller_type)
        .map(|dev| dev.clone())
}
