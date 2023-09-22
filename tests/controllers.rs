mod common;
use common::*;

use brewdrivers::controllers::*;
use brewdrivers::Result;

#[tokio::test]
async fn test_cn7500_controller() -> Result<()> {
    // Only run this test if we have a CN7500 in the testing config file, ie. we have one
    // physically connected.
    let dev = get_device_from_configuration(Controller::CN7500);
    if dev.is_none() {
        return Ok(());
    }

    let res = CN7500::from_device(dev.unwrap()).await;
    assert!(res.is_ok());
    let mut cn = res?;
    assert!(cn.get_pv().await? > 0.0);
    assert!(cn.set_sv(145.5).await.is_ok());
    assert_eq!(cn.get_sv().await?, 145.5);
    assert!(cn.run().await.is_ok());
    assert!(cn.stop().await.is_ok());

    Ok(())
}

#[test]
fn test_str1_controller() -> Result<()> {
    // Only run this test if we have a STR1 in the testing config file, ie. we have one
    // physically connected.
    let dev = get_device_from_configuration(Controller::STR1);
    if dev.is_none() {
        return Ok(());
    }

    let res = STR1::try_from(&dev.unwrap());
    assert!(res.is_ok());
    let mut str1 = res?;

    assert!(str1.connected().is_ok());
    assert!(str1.set_relay(0, BinaryState::On).is_ok());
    assert_eq!(str1.get_relay(0)?, BinaryState::On);

    assert!(str1.set_relay(0, BinaryState::Off).is_ok());
    assert_eq!(str1.get_relay(0)?, BinaryState::Off);

    assert!(str1.relay_count()? > 7);

    Ok(())
}

#[test]
fn test_wavesharev2_controller() -> Result<()> {
    // Only run this test if we have a WaveshareV2 in the testing config file, ie. we have one
    // physically connected.
    let dev = get_device_from_configuration(Controller::WaveshareV2);
    if dev.is_none() {
        return Ok(());
    }

    let res = WaveshareV2::try_from(&dev.unwrap());
    assert!(res.is_ok());
    let mut ws2 = res?;

    assert!(ws2.connected().is_ok());
    assert!(ws2.set_relay(0, BinaryState::On).is_ok());
    assert_eq!(ws2.get_relay(0)?, BinaryState::On);

    assert!(ws2.set_relay(0, BinaryState::Off).is_ok());
    assert_eq!(ws2.get_relay(0)?, BinaryState::Off);

    assert_eq!(ws2.software_revision()?, "v2.00");

    Ok(())
}

#[test]
fn test_waveshare_controller() -> Result<()> {
    // Only run this test if we have a Waveshare in the testing config file, ie. we have one
    // physically connected.
    let dev = get_device_from_configuration(Controller::Waveshare);
    if dev.is_none() {
        return Ok(());
    }

    let res = Waveshare::try_from(&dev.unwrap());
    assert!(res.is_ok());
    let mut ws1 = res?;

    assert!(ws1.connected().is_ok());
    assert!(ws1.set_relay(0, BinaryState::On).is_ok());
    assert_eq!(ws1.get_relay(0)?, BinaryState::On);

    assert!(ws1.set_relay(0, BinaryState::Off).is_ok());
    assert_eq!(ws1.get_relay(0)?, BinaryState::Off);

    assert_eq!(ws1.software_revision()?, "v1.00");

    Ok(())
}

