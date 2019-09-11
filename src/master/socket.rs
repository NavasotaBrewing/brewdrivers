// use ws::{connect, CloseCode};

// use crate::master::configuration::Configuration;

// pub fn run(config: Configuration) {
//     println!("Running master socket");

//     for rtu in config.RTUs() {
//         connect(format!("{}", rtu.ipv4()), |out| {
//             out.send("Send the config !");

//             move |msg| {
//                 println!("{}", msg);
//                 out.close(CloseCode::Normal)
//             }
//         }).expect("Something went wrong");

//     }

// }
