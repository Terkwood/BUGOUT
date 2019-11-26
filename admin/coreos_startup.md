# configuring coreos startup script for big box

Create a systemd init script

```sh
sudo vim /etc/systemd/system/bugout.service
```

Give it some values

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

Enable it

```sh
sudo systemctl enable /etc/systemd/system/bugout.service
sudo systemctl start bugout.service
```

Check the output

```sh
sudo journalctl -u bugout.service
```
