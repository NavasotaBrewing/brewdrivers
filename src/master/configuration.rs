// For RTUs
#![allow(non_snake_case)]
use std::net::SocketAddrV4;
use std::fs;

use serde::{Serialize, Deserialize};

use crate::RTU::relays::State;
// use crate::RTU::relays::{STR1, Board};
use crate::RTU::*;


fn get_rtu_id() -> String {
    String::from(fs::read_to_string("/rtu_id").expect("Couldn't read RTU id").trim())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Mode {
    Write,
    Read
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Driver {
    STR1,
    Omega,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Device {
    pub driver: Driver,
    pub name: String,
    pub id: String,
    pub state: State,
    pub addr: u8,
    pub controller_addr: u8,
    pub pv: Option<f64>,
    pub sv: Option<f64>,

}

impl Device {
    pub fn update(device: &mut Device, mode: &Mode) {
        match device.driver {
            Driver::STR1 => {
                let mut board = STR1::with_address(device.controller_addr);
                match mode {
                    Mode::Write => board.set_relay(device.addr, device.state.clone()),
                    Mode::Read => {}
                }
                // Read|Update
                device.state = board.get_relay(device.addr);
            },
            Driver::Omega => {
                let cn7500 = CN7500::new(device.controller_addr, "/dev/ttyAMA0", 19200);

                match mode {
                    Mode::Write => {
                        cn7500.set_sv(device.sv.unwrap());
                        match device.state {
                            State::On => cn7500.run(),
                            State::Off => cn7500.stop(),
                        }
                    },
                    Mode::Read => {}
                }

                // Read|Update
                device.pv = Some(cn7500.get_pv());
		device.sv = Some(cn7500.get_sv());
                if cn7500.is_running() {
                    device.state = State::On;
                } else {
                    device.state = State::Off;
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RTU {
    pub name: String,
    pub location: String,
    pub id: String,
    pub ipv4: SocketAddrV4,
    pub devices: Vec<Device>
}

impl RTU {
    pub fn from(json_string: &str) -> Result<RTU, String> {
        match serde_json::from_str::<RTU>(json_string) {
            Ok(rtu) => return Ok(rtu),
            Err(e) => return Err(format!("Could not deserialize json string to RTU: {}", e)),
        }
    }

    pub fn stringify(&self) -> Result<String, String> {
        match serde_json::to_string(&self) {
            Ok(rtu_string) => return Ok(rtu_string),
            Err(e) => return Err(format!("Could not stringify RTU: {}", e)),
        }
    }

    pub fn update(rtu: &mut RTU, mode: &Mode) {
        for mut device in &mut rtu.devices {
            Device::update(&mut device, &mode);
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Configuration {
    pub name: String,
    pub description: Option<String>,
    pub slackChannel: Option<String>,
    pub slackWebhook: Option<String>,
    pub date: Option<String>,
    pub mode: Mode,
    pub id: u32,
    pub RTUs: Vec<RTU>
}

impl Configuration {
    pub fn from(json_string: &str) -> Result<Configuration, String> {
        match serde_json::from_str::<Configuration>(json_string) {
            Ok(config) => return Ok(config),
            Err(e) => return Err(format!("Could not deserialize json string to configuration: {}", e)),
        }
    }

    pub fn stringify(&self) -> String {
        serde_json::to_string(&self).expect("Could not serialize configuration")
    }

    pub fn update(source_config: &Configuration, mode: &Mode) -> Configuration {
        // let mut config = Configuration::from(&config_string).expect("Couldn't deserialize config");
        let mut config = source_config.clone();

        for mut rtu in &mut config.RTUs {
            // Only update this RTU
            // println!("get_rtu_id(): {}", get_rtu_id());
            // println!("rtu.id: {}", rtu.id);
            if rtu.id == get_rtu_id() {
            	RTU::update(&mut rtu, &mode);
            }
        }
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_and_deserialize_a_configuration() {
        let data = r#"
        {
            "name": "My configuration",
            "description": "Some configuration i made to brew in march or something idk",
            "id": 45,
            "mode": "Write",
            "RTUs": [
                {
                "name": "Main Valves",
                "location": "Over there",
                "id": "1kjhsmdnbfaskudf687234",
                "ipv4": "192.168.0.34:3012",
                "devices": [
                    {
                        "driver": "STR1",
                        "addr": 0,
                        "controller_addr": 243,
                        "name": "some valve or something",
                        "state": "On",
                        "id": "s3h4ma8itu1lhfxcee"
                    },
                    {
                        "driver": "Omega",
                        "name": "RIMS PID",
                        "addr": 0,
                        "pv": 167.4,
                        "controller_addr": 0,
                        "id": "j12k3jhgsdkhfgj2h4bv4mnrb",
                        "sv": 172.0,
                        "state": "On"
                    }
                ]
                }
            ]
        }
        "#;

        let config = serde_json::from_str::<Configuration>(&data.trim()).expect("Couldn't deserialize configuration package");
        assert_eq!(config.name, String::from("My configuration"));
        let config_string = serde_json::to_string_pretty(&config).expect("Could not serialize configuration");
        println!("{}", config_string);
    }

    #[test]
    fn minimal_configuration() {
        let data = r#"{
            "name": "Minimal Configuration",
            "description": "This is a minimal configuration, all data here is required",
            "mode": "Read",
            "id": 32,
            "RTUs": []
        }"#;

        let config = serde_json::from_str::<Configuration>(&data).unwrap();
        assert_eq!(config.name, String::from("Minimal Configuration"));
    }

    #[test]
    fn config_from_from() {
        let config_string = r#"{
            "name": "My configuration",
            "description": "Some configuration i made to brew in march or something idk",
            "id": 13,
            "mode": "Write",
            "RTUs": [
                {
                "name": "Main Valves",
                "location": "Over there",
                "id": "1kjhsmdnbfaskudf687234",
                "ipv4": "0.0.0.0:3012",
                "devices": [
                    {
                        "driver": "STR1",
                        "addr": 0,
                        "controller_addr": 243,
                        "name": "some valve or something",
                        "state": "On",
                        "id": "s3h4ma8itu1lhfxcee"
                    },
                    {
                        "driver": "Omega",
                        "name": "RIMS PID",
                        "addr": 0,
                        "pv": 167.4,
                        "controller_addr": 0,
                        "id": "j12k3jhgsdkhfgj2h4bv4mnrb",
                        "sv": 172.0,
                        "state": "On"
                    }
                ]
                }
            ]
        }"#;

        let config = Configuration::from(config_string).expect("Something went wrong :(");
        assert_eq!(config.name, String::from("My configuration"));
        assert_eq!(config.RTUs.len(), 1);
    }
}
