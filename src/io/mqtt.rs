extern crate paho_mqtt as mqtt;
use crate::paser::f6::{F6Received, F6};
use crate::io::OutProcesser;
use chrono::Local;
use crossbeam_channel::Receiver;


pub struct MqttProcesser {
    host: String,
    clientid: String,
    username: String,
    password: String,
}

// impl MqttProcesser {
//     fn new() -> MqttProcesser {
//         let create_opts = mqtt::CreateOptionsBuilder::new()
//         .server_uri(self.host.clone())
//         .client_id(self.clientid.clone())
//         .finalize();
//     }

// }


impl OutProcesser for MqttProcesser {
    fn recv_f6_process(&self, receiver: &Receiver<F6>){
        let create_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri(self.host.clone())
            .client_id(self.clientid.clone())
            .finalize();
        
        let mut cli = mqtt::Client::new(create_opts).unwrap();
        
        // Initialize the consumer before connecting.
        // let rx = cli.start_consuming();
        
        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            // .keep_alive_interval(Duration::from_secs(20))
            .clean_session(true)
            .user_name(self.username.clone())
            .password(self.password.clone())
            .finalize();

        // Connect and wait for it to complete or fail.
        if let Err(e) = cli.connect(conn_opts) {
            log::error!("Unable to connect:\n\t{:?}", e);
        }

        loop {
            let f6 = receiver.recv().unwrap();
            let f6rec = F6Received {
                f6: f6,
                received: Local::now().to_rfc3339(),
            };
            let f6_serialized = serde_json::to_string(&f6rec).unwrap();
            let msg = mqtt::Message::new("f6", f6_serialized, 0);
            let tok = cli.publish(msg);
    
            if let Err(e) = tok {
                log::error!("Error sending message: {:?}", e);
            }
        }

    }

}

