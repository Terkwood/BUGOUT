# FCOS admin support

We have packer files, some example bash scripts, you name it.

## Where is the most recent FCOS version shown?

[Look at their downloads page](https://getfedora.org/coreos/download).

## Weird Trivia: Disable rpm-ostreed During Packer Build

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
