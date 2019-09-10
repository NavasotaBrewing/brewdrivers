use std::net::SocketAddrV4;

// use ws::listen;
use serde::{Serialize, Deserialize};

use crate::relays::State;

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
    state: u8,
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
struct Configuration {
    name: String,
    description: String,
    id: String,
    RTUs: Vec<RTU>
}


// impl Configuration {
// }

// Wait for connections
pub fn run() {
    let data = r#"
    {
        "name": "My configuration",
        "description": "Some configuration i made to brew in march or something idk",
        "id": "j3jhsmdnbk23j4gdf6872123",
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
                "state": 1,
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
                "state": 1
                }
            ]
            }
        ]
    }
    "#;

    let config = serde_json::from_str::<Configuration>(&data.trim());
    println!("{:?}", config);
    // println!("Waiting for connections...");
    // listen("0.0.0.0:3012", |out| {
    //     println!("Connected.");
    //     move |msg: ws::Message| {
    //         match msg.as_text() {
    //             Ok(text_data) => {
    //                 let config = serde_json::from_str::<Configuration>(&text_data);
    //                 println!("{:?}", config);
    //             },
    //             Err(e) => panic!(e),
    //         }
    //         out.send("Received")
    //     }
    // }).unwrap()
}
