// use ws::listen;
use crate::configuration::Configuration;
use serde_json;

pub fn run() {
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
    println!("{:?}", config);
    let config_string = serde_json::to_string_pretty(&config).expect("Could not serialize configuration");
    println!("{}", config_string);
    // Wait for connections
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
