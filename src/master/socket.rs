use ws::{connect, CloseCode, Message};
use crate::master::configuration::Configuration;

// Send the config to all RTUs in the config, and get it back
// fn propogate(config: &mut Configuration) -> &Configuration {
//     if config.RTUs.len() == 0 {
//         return config;
//     }

//     for rtu in &config.RTUs {
//         let ip = format!("{}", rtu.ipv4);

//         connect(ip.as_str(), |out| {
//             let config_string = config.stringify().expect("Could not stringify config");

//             // send config
//             let result = out.send(config_string);
//             match result {
//                 Ok(_) => {},
//                 Err(e) => println!("Error: could not propogate to server: {}", e)
//             }

//             move |msg: Message| {
//                 let mut new_config = Configuration::from(&msg.into_text().expect("Couldn't convert RTU reply to text"));
//                 config = &mut new_config.expect("Couldn't deserialize configuration");
//                 out.close(CloseCode::Normal)
//             }
//         }).expect("Could not connect");
//     }

//     config
// }

// pub fn run() {
//     // Open a port and listen for connections from the frontend
//     listen("0.0.0.0:1612", |frontend| {
//         // Listen for a message
//         move |msg: Message| {
//             // Once a message it received, pull a config out of it
//             let config = Configuration::from(msg.as_text()
//                                             .expect("Couldn't get text from socket package"))
//                                             .expect("Couldn't deserialize configuration");
//             // send that config to all RTUs
//             for rtu in &config.RTUs {
//                 let ip = format!("{}", rtu.ipv4);

//             }
//         }
//     })
// }



// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn propogate_config() {
//         let config_string = r#"{
//             "name": "My configuration",
//             "description": "Some configuration i made to brew in march or something idk",
//             "id": "j3jhsmdnbk23j4gdf6872123",
//             "mode": "Write",
//             "RTUs": [
//                 {
//                 "name": "Main Valves",
//                 "location": "Over there",
//                 "id": "1kjhsmdnbfaskudf687234",
//                 "ipv4": "0.0.0.0:3012",
//                 "devices": [
//                     {
//                         "driver": "STR1",
//                         "addr": 0,
//                         "controller_addr": 243,
//                         "name": "some valve or something",
//                         "state": "On",
//                         "id": "s3h4ma8itu1lhfxcee"
//                     },
//                     {
//                         "driver": "Omega",
//                         "name": "RIMS PID",
//                         "addr": 0,
//                         "pv": 167.4,
//                         "controller_addr": 0,
//                         "id": "j12k3jhgsdkhfgj2h4bv4mnrb",
//                         "sv": 172.0,
//                         "state": "On"
//                     }
//                 ]
//                 }
//             ]
//         }"#;
//         let mut config = Configuration::from(config_string).unwrap();

//         let new_config = propogate(&mut config);
//         assert_eq!(new_config.id, config.id);
//     }
// }
