# FCOS admin support

We have packer files, some example bash scripts, you name it.

## Using packer to build the VM image

You need to [install packer](https://learn.hashicorp.com/tutorials/packer/get-started-install-cli) on a local machine which will drive the deployment.

You should create a directory which will contain implementations of the example scripts, and a directory where you can store `.env` files for each service:

```sh
mkdir -p /path/to/bugout/dev
mkdir -p /path/to/bugout/dev/env-files
```

You need a valid `vpc-id` and `subnet-id` for your AWS instance. Copy [set_vpc_subnet_env.example.sh](./set_vpc_subnet_env.example.sh) and fill in these values.

You should use the [gateway packer file](gateway-packer.json), as is. You do not need to make a copy of this file, but it will rely on your env variables (including AWS secrets!) to be set correctly. Usage of this packer file is demonstrated in [pack-example.sh](pack-example.sh).

You must create FCOS ignition file, as in [gateway-example.yaml](gateway-example.yaml).

## Launching the instance

You can use [launch.sh](launch.sh) to launch the instance. You need to have several env vars sourced to make it work:

```sh
TEMPLATE_ID=xyz \
AMI_ID=abc \
SUBNET_ID=def \
AZ=availzone00wat \
sh launch.sh
```

## Trivia

### Where is the most recent FCOS version shown?

[Look at their downloads page](https://getfedora.org/coreos/download).

## Disable rpm-ostreed During Packer Build

If you take a look [here](https://github.com/Terkwood/BUGOUT/pull/295) and [here](https://github.com/coreos/rpm-ostree/issues/1692#issuecomment-443215317) you'll find that the `systemctl stop rpm-ostreed` command in the packer.json files for gateway and kafka is necessary to prevent a transient error.

e.g. in [gateway-packer.json](gateway-packer.json):

```json
    {
      "type": "shell",
      "inline": [
        "sudo systemctl stop rpm-ostreed",
        "sudo rpm-ostree upgrade",
        "sudo rpm-ostree install htop",
        "sudo rpm-ostree install tmux"
      ]
    },
```

What's more `rpm-ostreed` controls [automatic updates](https://docs.fedoraproject.org/en-US/iot/applying-updates-UG/#_automatic_updates) and needs to be disabled to prevent havoc during the `packer build` step. You don't want the machine rebooting during your 79-hour `cargo install` step!
