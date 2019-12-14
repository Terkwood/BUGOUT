# config procedure

Create a systemd init script

```sh
sudo vim /etc/systemd/system/bugout.service
```

...Fill in values (see below)...

Enable it

```sh
sudo systemctl enable /etc/systemd/system/bugout.service
sudo systemctl start bugout.service
```

Check the output

```sh
sudo journalctl -u bugout.service
```

## example systemd startup script for gateway box

```text
[Unit]
Description=BUGOUT
After=docker.service
Requires=docker.service

[Service]
ExecStart=/usr/bin/sh /home/core/BUGOUT/admin/start-gateway-host.sh

[Install]
WantedBy=multi-user.target
```

## example systemd startup script for kafka box

```text
[Unit]
Description=BUGOUT
After=docker.service
Requires=docker.service

[Service]
ExecStart=/usr/bin/sh /home/core/BUGOUT/admin/start-kafka-host.sh

[Install]
WantedBy=multi-user.target
```

## Configuring Redis in CoreOS

Redis requires three tweaks to kernel config in CoreOS: Transparent Huge Pages disabled, overcommmit_memory enabled, and the TCP backlog raised.

These configs are managed with a [systemd
script](config-redis.service), which calls out to a
small [shell script](config-redis.sh).
