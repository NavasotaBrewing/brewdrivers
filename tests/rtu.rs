use brewdrivers::{
    controllers::Controller, drivers::InstrumentError, model::RTU, state::BinaryState,
};

#[tokio::test]
async fn test_generate_and_update_device_state() -> Result<(), InstrumentError> {
    // This generates an RTU state from the config file and updates all state
    // values in all devices. It's like taking a snapshot of all state values for the whole RTU.
    let res = RTU::generate(Some(brewdrivers::defaults::test_config_file()));
    assert!(res.is_ok());
    let mut rtu = res.unwrap();
    assert!(rtu.devices.len() > 0);

    let relay = rtu
        .devices
        .iter_mut()
        .find(|dev| *dev.conn.controller() == Controller::WaveshareV2);

    if let Some(device) = relay {
        // All state values start as None
        assert!(device.state.relay_state.is_none());
        assert!(device.state.pv.is_none());
        assert!(device.state.sv.is_none());

        device.update().await?;

        assert!(device.state.relay_state.is_some());
        assert!(device.state.pv.is_none());
        assert!(device.state.sv.is_none());
    }

    let pid = rtu
        .devices
        .iter_mut()
        .find(|dev| *dev.conn.controller() == Controller::CN7500);

    if let Some(device) = pid {
        // All state values start as None
        assert!(device.state.relay_state.is_none());
        assert!(device.state.pv.is_none());
        assert!(device.state.sv.is_none());

        device.update().await?;

        assert!(device.state.relay_state.is_some());
        assert!(device.state.pv.is_some());
        assert!(device.state.sv.is_some());
    }

    Ok(())
}

#[tokio::test]
async fn test_device_enact() -> Result<(), InstrumentError> {
    // There does exist an RTU::enact() method, but we try not to use it
    // because it will try to write to every device. This takes a long time.
    let res = RTU::generate(Some(brewdrivers::defaults::test_config_file()));
    assert!(res.is_ok());
    let mut rtu = res.unwrap();
    assert!(rtu.devices.len() > 0);

    let relay = rtu
        .devices
        .iter_mut()
        .find(|dev| *dev.conn.controller() == Controller::WaveshareV2);

    // All state values start as None
    if let Some(device) = relay {
        assert!(device.state.relay_state.is_none());
        assert!(device.state.pv.is_none());
        assert!(device.state.sv.is_none());
        device.state.relay_state = Some(BinaryState::On);
        device.enact().await?;
    }

    let relay_again = rtu
        .devices
        .iter_mut()
        .find(|dev| *dev.conn.controller() == Controller::WaveshareV2);
    if let Some(device) = relay_again {
        assert_eq!(device.state.relay_state, Some(BinaryState::On));
        assert!(device.state.pv.is_none());
        assert!(device.state.sv.is_none());

        device.state.relay_state = Some(BinaryState::Off);
        device.enact().await?;
    }

    Ok(())
}
