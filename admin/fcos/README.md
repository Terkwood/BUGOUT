# FCOS admin support

We have packer files, some example bash scripts, you name it.

## Weird Packer Trivia

If you take a look [here](https://github.com/Terkwood/BUGOUT/pull/295) and [here](https://github.com/coreos/rpm-ostree/issues/1692#issuecomment-443215317) you'll find that the `systemctl restart rpm-ostreed` command in the packer.json files for gateway and kafka is necessary to prevent a transient error.

e.g. in [gateway-packer.json](gateway-packer.json):
```json
    {
      "type": "shell",
      "inline": [
        "sudo systemctl restart rpm-ostreed",
        "sudo rpm-ostree upgrade",
        "sudo rpm-ostree install htop",
        "sudo rpm-ostree install tmux"
      ]
    },
```
