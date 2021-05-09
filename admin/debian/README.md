# debian cloud config

you should symlink the `*.service` files from this directory, into your working directory for dev / prod deployment

```sh
ln -s path/to/BUGOUT/admin/debian/*.service .
```

these will copied by packer into `/etc/systemd/system`
