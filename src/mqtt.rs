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

pub struct MQTT_Client {
    client: mqtt::AsyncClient,
    active: bool,
}

// -chat/channels/..
impl MQTT_Client {
    pub fn new(id: String) -> Self {
        let host = "tcp://localhost:1883".to_string();
        let create_options = mqtt::CreateOptionsBuilder::new()
            .server_uri(host)
            .client_id(id.to_string())
            .user_data(Box::new(RwLock::new("chat/channels/default")))
            .finalize();
        let mut client = mqtt::AsyncClient::new(create_options).unwrap();
        client.set_connected_callback(|_client: &mqtt::AsyncClient| {
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
        }
    }

    pub fn connect(&mut self) {
        let message = mqtt::Message::new(
            "chat/channels/default",
            "Async subscriber lost connection",
            1,
        );

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
        thread::spawn(move || {
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
