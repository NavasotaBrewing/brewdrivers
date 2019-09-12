// For RTUs
#![allow(non_snake_case)]
use std::net::SocketAddrV4;

use serde::{Serialize, Deserialize, Deserializer, Serializer};

use crate::RTU::relays::State;

#[derive(Debug)]
pub enum Mode {
    Write,
    Read
}

impl Serialize for Mode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Mode::Write => serializer.serialize_unit_variant("write", 0, "Write"),
            Mode::Read => serializer.serialize_unit_variant("read", 1, "Read"),
        }
    }
}

impl<'de> Deserialize<'de> for Mode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "write" => Mode::Write,
            "read" => Mode::Read,
            _ => Mode::Read,
        })
    }
}

#[derive(Debug)]
pub enum Driver {
    STR1,
    Omega,
}

impl Serialize for Driver {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Driver::STR1 => serializer.serialize_unit_variant("STR1", 0, "STR1"),
            Driver::Omega => serializer.serialize_unit_variant("omega", 1, "Omega"),
        }
    }
}

impl<'de> Deserialize<'de> for Driver {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "STR1" => Driver::STR1,
            "omega" => Driver::Omega,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub driver: Driver,
    pub name: String,
    pub id: String,
    pub state: State,
    pub addr: u8,
    pub controller_addr: u8
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RTU {
    pub name: String,
    pub location: String,
    pub id: String,
    pub ipv4: SocketAddrV4,
    pub devices: Vec<Device>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub name: String,
    pub description: String,
    pub mode: Mode,
    pub id: String,
    pub RTUs: Vec<RTU>
}

impl Configuration {
    pub fn from(json_string: &str) -> Result<Configuration, String> {
        match serde_json::from_str::<Configuration>(json_string) {
            Ok(config) => return Ok(config),
            Err(e) => return Err(format!("Could not deserialize json string to configuration: {}", e)),
        }
    }

    pub fn stringify(&self) -> Result<String, String> {
        match serde_json::to_string(&self) {
            Ok(config_string) => return Ok(config_string),
            Err(e) => return Err(format!("Could not stringify config: {}", e)),
        }
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
            "id": "j3jhsmdnbk23j4gdf6872123",
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
            "id": "someuniqueid1234",
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
            "id": "j3jhsmdnbk23j4gdf6872123",
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
