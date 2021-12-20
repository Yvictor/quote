extern crate paho_mqtt as mqtt;
use crate::io::OutProcesser;
use crate::paser::f6::{F6Received, F6};
use chrono::Local;
use crossbeam_channel::{Sender, Receiver, bounded};
use bus::BusReader;
use std::thread;

pub struct MqttOutProcesser {
    host: String,
    clientid: String,
    username: String,
    password: String,
    // client: mqtt::AsyncClient,
    sender: Sender<F6>,
    // receiver: Receiver<F6>,
    threads: Vec<thread::JoinHandle<()>>,
}

pub struct MqttWorker {
    receiver: Receiver<F6>,
    client: mqtt::AsyncClient,
}


fn new_client(host: &str, clientid: &str, username: &str, password: &str) -> mqtt::AsyncClient{
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
    cli
}

impl MqttWorker {
    pub fn new(receiver: Receiver<F6>, host: &str, clientid: &str, username: &str, password: &str) -> MqttWorker{
        let cli = new_client(host, clientid, username, password);
        MqttWorker{receiver: receiver, client: cli}
    }

    pub fn start(&mut self) {
        let mut count = 0;
        loop {    
            let f6 = self.receiver.recv().unwrap();
            let f6rec = F6Received {
                f6: f6,
                received: Local::now().to_rfc3339(),
            };
            if count == 0 {
                count = f6rec.f6.header.no;
            } else {
                count += 1;
            }
            if count != f6rec.f6.header.no {
                log::error!("count: {}, no: {}", count, f6rec.f6.header.no);
                count = f6rec.f6.header.no;
            }
            let f6_serialized = serde_json::to_string(&f6rec).unwrap();
            let msg = mqtt::Message::new("f6", f6_serialized, 0);
            let _tok = self.client.publish(msg);
    
        }
    }
}


impl MqttOutProcesser {
    pub fn new(host: &str, clientid: &str, username: &str, password: &str, n: usize) -> MqttOutProcesser {
        // let cli = new_client(host, clientid, username, password);
        let (sender, receiver): (Sender<F6>, Receiver<F6>) = bounded(4096);
        let mut threads = Vec::with_capacity(n);
        for _ in 0..n {
            let mut worker = MqttWorker::new(receiver.clone(), host, clientid, username, password);
            let thread = thread::spawn(move || {
               worker.start()
            });
            threads.push(thread);
        }
        MqttOutProcesser {
            host: String::from(host),
            clientid: String::from(clientid),
            username: String::from(username),
            password: String::from(password),
            sender: sender,
            threads: threads,
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
        // self.client = cli;
    }
}

impl OutProcesser for MqttOutProcesser {
    fn recv_f6_process(&mut self, receiver: &mut BusReader<F6>) {
        let mut count = 0;
        loop {
            let f6 = receiver.recv().unwrap();
            if count == 0 {
                count = f6.header.no;
            } else {
                count += 1;
            }
            if count != f6.header.no {
                log::error!("count: {}, no: {}", count, f6.header.no);
                count = f6.header.no;
            }
            match self.sender.send(f6) {
                Ok(_) => (),
                Err(e) => println!("sender error: {:?}", e),
            }
        }
    }
}
