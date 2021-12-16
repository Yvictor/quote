extern crate paho_mqtt as mqtt;
use crate::io::OutProcesser;
use crate::paser::f6::{F6Received, F6};
use chrono::Local;
use crossbeam_channel::Receiver;

pub struct MqttOutProcesser {
    host: String,
    clientid: String,
    username: String,
    password: String,
    client: mqtt::AsyncClient,
}

impl MqttOutProcesser {
    pub fn new(host: &str, clientid: &str, username: &str, password: &str) -> MqttOutProcesser {
        let create_opts = mqtt::CreateOptionsBuilder::new()
            .mqtt_version(mqtt::MQTT_VERSION_5)
            .server_uri(host.clone())
            .client_id(clientid.clone())
            .finalize();
        let cli = mqtt::AsyncClient::new(create_opts).unwrap();

        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .mqtt_version(mqtt::MQTT_VERSION_5)
            .user_name(username.clone())
            .password(password.clone())
            .finalize();
        if let Err(e) = cli.connect(conn_opts).wait() {
            log::error!("Unable to connect:\n\t{:?}", e);
        }
        MqttOutProcesser {
            host: String::from(host),
            clientid: String::from(clientid),
            username: String::from(username),
            password: String::from(password),
            client: cli,
        }
    }

    pub fn reset_client(&mut self) {
        let create_opts = mqtt::CreateOptionsBuilder::new()
            .mqtt_version(mqtt::MQTT_VERSION_5)
            .server_uri(self.host.clone())
            .client_id(self.clientid.clone())
            .finalize();
        let cli = mqtt::AsyncClient::new(create_opts).unwrap();

        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .mqtt_version(mqtt::MQTT_VERSION_5)
            .user_name(self.username.clone())
            .password(self.password.clone())
            .finalize();

        if let Err(e) = cli.connect(conn_opts).wait() {
            log::error!("Unable to connect:\n\t{:?}", e);
        }
        self.client = cli;
    }
}

impl OutProcesser for MqttOutProcesser {
    fn recv_f6_process(&mut self, receiver: &Receiver<F6>) {
        loop {
            let f6 = receiver.recv().unwrap();
            let f6rec = F6Received {
                f6: f6,
                received: Local::now().to_rfc3339(),
            };
            let f6_serialized = serde_json::to_string(&f6rec).unwrap();
            let msg = mqtt::Message::new("f6", f6_serialized, 0);
            let _tok = self.client.publish(msg);
            // if let Err(e) = tok.wait() {
            //     log::error!("Error sending message: {:?}", e);
            // }
        }
    }
}
