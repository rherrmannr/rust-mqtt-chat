mod chat;
mod mqtt;
mod user;

use std::env;

fn main() {
    let mut id: String = String::from("");
    for arg in env::args() {
        id = arg;
    }
    let mut client = mqtt::MqttClient::new(id);
    client.connect();
    client.run();
    loop {}
}
