// dynamic mqtt handler for subscribe and publish

use std::{
    io,
    sync::RwLock,
    thread::{self},
    time::Duration,
};

use futures::executor::block_on;

use mqtt::AsyncClient;
use paho_mqtt as mqtt;

fn on_connect_success_callback(client: &mqtt::AsyncClient, _: u16) {
    // chat.notify(Notification::Success);
    println!("connected. try to subscirbe to chat/channels/default");
    client.subscribe("chat/channels/default", 0);
}

fn on_connect_failure_callback(client: &mqtt::AsyncClient, _: u16, error_code: i32) {
    // chat.notify(Notifcation::Failure(error_code));
    println!("failed to connect. retry in 1 second");
    thread::sleep(Duration::from_millis(1000));
    client.reconnect_with_callbacks(on_connect_success_callback, on_connect_failure_callback);
}

#[derive(Clone)]
pub struct MqttClient {
    client: mqtt::AsyncClient,
    active: bool,
    id: String,
}

// -chat/channels/..
impl MqttClient {
    pub fn new(id: String) -> Self {
        let host = "tcp://localhost:1883".to_string();
        let clone = id.clone();
        let create_options = mqtt::CreateOptionsBuilder::new()
            .server_uri(host)
            .client_id(&clone.to_string())
            .user_data(Box::new(RwLock::new("chat/channels/default")))
            .finalize();
        let mut client = mqtt::AsyncClient::new(create_options).unwrap();
        client.set_connected_callback(move |client: &mqtt::AsyncClient| {
            let mut payload = "user joined: ".to_owned();
            payload.push_str(&clone.to_owned());
            let message = mqtt::Message::new("chat/channels/default", payload, mqtt::QOS_1);
            client.publish(message);
            println!("connected");
            // chat.notify(Notification::Connected);
        });
        client.set_connection_lost_callback(|client: &AsyncClient| {
            // chat.notify(Notification::Lost);
            println!("connection lost. retry in 1 second");
            thread::sleep(Duration::from_millis(1000));
            client
                .reconnect_with_callbacks(on_connect_success_callback, on_connect_failure_callback);
        });
        client.set_message_callback(|_, message| {
            // TODO: handle different channels..
            if let Some(message) = message {
                let topic = message.topic();
                if topic == "chat/channels/default" {
                    // chat.notify(Notification::Message(message.payload_str()));
                    println!("received : {}", message.payload_str());
                }
            }
        });
        Self {
            client,
            active: false,
            id: id,
        }
    }

    pub fn connect(&mut self) {
        // playload=
        let mut payload = "user left: ".to_owned();
        payload.push_str(&self.id);
        let message = mqtt::Message::new("chat/channels/default", payload, 1);

        let connect_options = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(20))
            .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
            .clean_session(true)
            .will_message(message)
            .finalize();
        println!("connecting to MQTT chat");
        self.client.connect_with_callbacks(
            connect_options,
            on_connect_success_callback,
            on_connect_failure_callback,
        );
    }

    pub fn run(mut self) {
        self.active = true;
        thread::spawn(move || loop {
            if !self.active {
                return;
            }
            if let Err(err) = block_on(async {
                println!("Read input");
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("Failed");
                let message = mqtt::Message::new("chat/channels/default", input, mqtt::QOS_1);
                self.client.publish(message).await?;

                Ok::<(), mqtt::Error>(())
            }) {
                eprintln!("{}", err);
            }
        });
    }
}
