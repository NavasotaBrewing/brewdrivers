// For RTUs
#![allow(non_snake_case)]
use std::net::SocketAddrV4;

use serde::{Serialize, Deserialize};

use crate::relays::State;

#[derive(Debug, Serialize, Deserialize)]
enum Mode {
    Write,
    Read
}

#[derive(Debug, Serialize, Deserialize)]
enum Driver {
    STR1,
    Omega,
}

#[derive(Debug, Serialize, Deserialize)]
struct Device {
    driver: Driver,
    name: String,
    id: String,
    state: State,
    addr: u8,
    controller_addr: u8
}

#[derive(Debug, Serialize, Deserialize)]
struct RTU {
    name: String,
    location: String,
    id: String,
    ipv4: SocketAddrV4,
    devices: Vec<Device>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    name: String,
    description: String,
    mode: Mode,
    id: String,
    RTUs: Vec<RTU>
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
}
