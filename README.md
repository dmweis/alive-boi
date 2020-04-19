# Alive Boi

Tiny rust node that sends MQTT messages on startup and shutdown

## Send messages on startup and shutdown of machine

### install binary

``` console
cargo install --git https://github.com/dmweis/alive_boi
```

### Add following file to `/ets/systemd/system/alive_boi.service`

``` ini
[Unit]
Description=Alive Boi

[Service]
Type=simple
Restart=on-failure
RestartSec=5s
ExecStart=/home/USER_NAME/.cargo/bin/alive_boi --mqtt_host mqtt.local --device_name DEVICE_NAME --topic TOPIC_NAME

[Install]
WantedBy=default.target
```

### Activate service

``` console
sudo systemctl daemon-reload
sudo systemctl enable alive_boi
sudo systemctl start alive_boi
```