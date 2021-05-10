# debian cloud config

you should symlink the `*.service` files from this directory, into your working directory for dev / prod deployment

```sh
ln -s path/to/BUGOUT/admin/debian/*.service .
```

these will copied by packer into `/etc/systemd/system`

you should also symlink your RSA pubkey into your working directory, so that the host can recognize you

```sh
ln -s ~/.ssh/id_rsa.pub .
```
