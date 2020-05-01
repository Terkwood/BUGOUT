For example, if you want to create an image for gateway box.

```sh
source /some/set_vpc_subnet_env.sh
USER_DATA_FILE=/some/box.ign \
	ENV_SRC=/path/to/local/BUGOUT
	packer build gateway-packer.json
```
