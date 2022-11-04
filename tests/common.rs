use brewdrivers::controllers::Controller;
use brewdrivers::model::{Device, RTU};


/// Deserializes the testing configuration and finds the first device in it with the
/// right controller type, if any
pub fn get_device_from_configuration(controller_type: Controller) -> Option<Device> {
    let rtu = RTU::generate(Some(brewdrivers::TEST_CONFIG_FILE)).unwrap();
    rtu.devices
        .iter()
        .find(|dev| *dev.conn.controller() == controller_type )
        .map( |dev|  dev.clone() )
}