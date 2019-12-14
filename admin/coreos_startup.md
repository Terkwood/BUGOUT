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

### Transparent Huge Pages

Redis requires Transparent Huge Pages to be disabled in
the kernel, via

```sh
echo never > /sys/kernel/mm/transparent_hugepage/enabled
```

Under CoreOS, this is managed with a [systemd
script](disable-thp.service), which calls out to a
small [shell script that disables THP](disable-thp.sh).

### overcommit_memory kernel module

Redis asks for the kernel module `overcommit_memory` to
be enabled.  To accomplish this, write a file `/etc/sysctl.d/overcommit_memory.conf`:

```text
vm.overcommit_memory = 1
```
