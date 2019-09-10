// use ws::listen;
// use crate::configuration::Configuration;
// use serde_json;

pub fn run() {
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
