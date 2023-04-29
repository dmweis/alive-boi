use anyhow::Context;
use chrono::prelude::*;
use clap::Parser;
use rumqtt::{LastWill, MqttClient, MqttOptions, Notification, QoS, ReconnectOptions};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::*;

#[derive(Serialize)]
enum Status {
    Alive,
    Reconnected,
    Dead,
}

#[derive(Serialize)]
struct AliveReport {
    unit_name: String,
    status: Status,
    start_time: DateTime<Local>,
}

#[derive(Parser, Debug)]
#[command(
    version,
    author = "David M. Weis <dweis7@gmail.com>",
    about = "Alive Boi"
)]
struct Cli {
    #[arg(long)]
    mqtt_host: String,
    #[arg(long)]
    device_name: String,
    #[arg(long)]
    topic: String,
    #[arg(long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();

    let args = Cli::parse();

    let _config = get_configuration(args.config)?;

    let start_time: DateTime<Local> = Local::now();

    let last_will_message = AliveReport {
        unit_name: args.device_name.clone(),
        status: Status::Dead,
        start_time,
    };

    let last_will = LastWill {
        topic: args.topic.clone(),
        message: serde_json::to_string(&last_will_message)
            .context("Failed to serialize last will message")?,
        qos: QoS::AtLeastOnce,
        retain: true,
    };

    let node_name = format!("alive_boi_{}", &args.device_name);
    let mqtt_options = MqttOptions::new(node_name, args.mqtt_host.clone(), 1883)
        .set_last_will(last_will)
        .set_keep_alive(5)
        .set_reconnect_opts(ReconnectOptions::Always(5));
    let (mut mqtt_client, notifications) =
        MqttClient::start(mqtt_options).expect("Failed to connect to MQTT host");

    let alive_message = AliveReport {
        unit_name: args.device_name.clone(),
        status: Status::Alive,
        start_time,
    };

    mqtt_client
        .publish(
            args.topic.clone(),
            QoS::AtLeastOnce,
            true,
            serde_json::to_string(&alive_message)
                .context("Failed to serialize last will message")?,
        )
        .expect("Failed to send alive message");

    for notification in notifications {
        match notification {
            Notification::Disconnection => {
                warn!("Client lost connection");
            }
            Notification::Reconnection => {
                warn!("client reconnected");
                let alive_message = AliveReport {
                    unit_name: args.device_name.clone(),
                    status: Status::Reconnected,
                    start_time,
                };
                mqtt_client
                    .publish(
                        args.topic.clone(),
                        QoS::AtLeastOnce,
                        true,
                        serde_json::to_string(&alive_message)
                            .context("Failed to serialize last will message")?,
                    )
                    .expect("Failed to send alive message");
            }
            other => {
                warn!("Uncontexted message {:?}", other);
            }
        }
    }
    Ok(())
}

pub fn setup_tracing() {
    tracing_subscriber::fmt()
        .pretty()
        .with_thread_names(true)
        .with_max_level(tracing::Level::INFO)
        .init();
}

pub fn get_configuration(config: Option<PathBuf>) -> anyhow::Result<MqttConfig> {
    let mut config_builder = config::Config::builder();

    if let Some(config) = config {
        info!("Using configuration from {:?}", config);
        config_builder = config_builder.add_source(config::File::with_name(
            config
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Failed to convert path"))?,
        ));
    } else {
        info!("Using dev configuration");
        config_builder = config_builder
            .add_source(config::File::with_name("configuration/settings"))
            .add_source(config::File::with_name("configuration/dev_settings"));
    }

    config_builder = config_builder.add_source(config::Environment::with_prefix("APP"));

    let config = config_builder.build()?;

    Ok(config.try_deserialize::<MqttConfig>()?)
}

const DEFAULT_MQTT_PORT: u16 = 1883;

const fn default_mqtt_port() -> u16 {
    DEFAULT_MQTT_PORT
}

#[derive(Deserialize, Debug, Clone)]
pub struct MqttConfig {
    pub broker_host: String,
    #[serde(default = "default_mqtt_port")]
    pub broker_port: u16,
    pub client_id: String,
}
