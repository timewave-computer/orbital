# local interchaintest

## setup

### install interchaintest

```bash
git clone --depth 1 --branch v8.3.0 https://github.com/strangelove-ventures/interchaintest; cd interchaintest; git switch -c v8.3.0
```

```bash
cd local-interchain
```

```bash
# NOTE: your binary will link back to this location of where you install.
# If you rename the folder or move it, you need to `make install` the binary again.
make install
```

### spinning up the env

```bash
just local-ic-start
```

### running tests

```bash
just local-ic-run
```
> make sure you have the neutron ICQ relayer docker image available on your machine prior to running the tests

# Neutron ICQ relayer setup

Somewhere on your machine, execute the following commands:

```sh
git clone git@github.com:neutron-org/neutron-query-relayer.git
cd neutron-query-relayer
make build-docker
```
