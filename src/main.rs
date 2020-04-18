use clap::{App, Arg};
use rumqtt::{LastWill, MqttClient, MqttOptions, QoS};
use serde::Serialize;
use chrono::prelude::*;

#[derive(Serialize)]
enum Status {
    Alive,
    Dead,
}

#[derive(Serialize)]
struct AliveReport {
    unit_name: String,
    status: Status,
    start_time: DateTime<Local>,
}

fn main() {
    let matches = App::new("Alive boi")
        .version("1.0")
        .author("David Weis <dweis7@gmail.com>")
        .about("Sends MQTT keep alive messages to host")
        .arg(
            Arg::with_name("mqtt_host")
                .long("mqtt_host")
                .help("points to hostname of MQTT host")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("device_name")
                .long("device_name")
                .help("Name under which the device should appear")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("topic")
                .long("topic")
                .help("Topic for alive and last will messages")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let mqtt_host = matches
        .value_of("mqtt_host")
        .expect("You must provide MQTT host");
    let device_name = matches
        .value_of("device_name")
        .expect("You must provide device_name");
    let topic = matches
        .value_of("topic")
        .expect("You must provide a topic name");


    let now: DateTime<Local> = Local::now();
    
    let last_will_message = AliveReport {
        unit_name: device_name.to_owned(),
        status: Status::Dead,
        start_time: now,
    };

    let last_will = LastWill {
        topic: topic.to_owned(),
        message: serde_json::to_string(&last_will_message)
            .expect("Failed to serialize last will message"),
        qos: QoS::AtLeastOnce,
        retain: false,
    };

    let node_name = format!("alive_boi_{}", &device_name);
    let mqtt_options = MqttOptions::new(node_name, mqtt_host, 1883)
        .set_last_will(last_will)
        .set_keep_alive(5);
    let (mut mqtt_client, notifications) =
        MqttClient::start(mqtt_options).expect("Failed to connect to MQTT host");

    let alive_message = AliveReport {
        unit_name: device_name.to_owned(),
        status: Status::Alive,
        start_time: now,
    };

    mqtt_client
        .publish(
            topic,
            QoS::AtLeastOnce,
            false,
            serde_json::to_string(&alive_message).expect("Failed to serialize last will message"),
        )
        .expect("Failed to send alive message");

    for notification in notifications {
        println!("Unexpected message {:?}", notification);
    }
}
