# OAX Parrot 

OAX Parachain 

## Features 

- [x] ERC20 Standard 
- [x] Atomic Swap (Single tx swap for ERC20 tokens) 
- [x] Multi-Transfer 
- [x] Automatic Fee Burn 
- [x] Fee Delegation

## Necessary Custom Types 

[Types.json](https://github.com/OAXFoundation/parrot/blob/master/js/parrot-client/src/types/types.json)

## Pre-reqs

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Initialize your Wasm Build environment:

```bash
./scripts/init.sh
```


## Running the Node 

1) `cargo build` (add `--release` for production build)
2) `./target/debug/parrot --alice --dev`
3) If you want to re-launch the chain, you can start fresh by purging it `./target/debug/parrot purge-chain --dev`

## Testing Custom Pallets

1) `cargo test`
2) To run individual test, you can run `cargo test MODULE` , so if you wanted to test the burn module you could run `cargo test burn`

## FrontEnd

Currently we do not have a custom front-end for this chain. You can use polkadot-js/apps as a front end for the chain without any issues. 

The last tested stable release: https://github.com/polkadot-js/apps/tree/v0.48.1


## Client + End to End Tests + Demos 

Instructions to run these tests and demos are in the [REAMDE](https://github.com/OAXFoundation/parrot/blob/master/js/README.md)



## Feature Implementations 

[ERC20 Standard](https://github.com/OAXFoundation/parrot/blob/master/pallets/prc20/src/lib.rs)

[Atomic Swap](https://github.com/OAXFoundation/parrot/blob/master/pallets/prc20/src/lib.rs#L220)

[Native Multi Transfer](https://github.com/OAXFoundation/parrot/blob/master/pallets/multi_transfer/src/lib.rs) 

[PRC20 Multi Transfer](https://github.com/OAXFoundation/parrot/blob/master/pallets/prc20/src/lib.rs#L234)

[Automatic Fee Burn](https://github.com/OAXFoundation/parrot/blob/master/pallets/burn/src/lib.rs)

[Fee Delegation](https://github.com/OAXFoundation/parrot/blob/master/pallets/delegation/src/lib.rs)



## Run

### Single Node Development Chain

Purge any existing developer chain state:

```bash
./target/release/parrot purge-chain --dev
```

Start a development chain with:

```bash
./target/release/parrot --dev
```

Detailed logs may be shown by running the node with the following environment variables set: `RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev`.

### Multi-Node Local Testnet

If you want to see the multi-node consensus algorithm in action locally, then you can create a local testnet with two validator nodes for Alice and Bob, who are the initial authorities of the genesis chain that have been endowed with testnet units.

Optionally, give each node a name and expose them so they are listed on the Polkadot [telemetry site](https://telemetry.polkadot.io/#/Local%20Testnet).

You'll need two terminal windows open.

We'll start Alice's substrate node first on default TCP port 30333 with her chain database stored locally at `/tmp/alice`. The bootnode ID of her node is `QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR`, which is generated from the `--node-key` value that we specify below:

```bash
cargo run -- \
  --base-path /tmp/alice \
  --chain=local \
  --alice \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --telemetry-url 'ws://telemetry.polkadot.io:1024 0' \
  --validator
```

In the second terminal, we'll start Bob's substrate node on a different TCP port of 30334, and with his chain database stored locally at `/tmp/bob`. We'll specify a value for the `--bootnodes` option that will connect his node to Alice's bootnode ID on TCP port 30333:

```bash
cargo run -- \
  --base-path /tmp/bob \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR \
  --chain=local \
  --bob \
  --port 30334 \
  --telemetry-url 'ws://telemetry.polkadot.io:1024 0' \
  --validator
```

Additional CLI usage options are available and may be shown by running `cargo run -- --help`.

### Run in Docker

First, install [Docker](https://docs.docker.com/get-docker/) and [Docker Compose](https://docs.docker.com/compose/install/).

Then run the following command to start a single node development chain.

```bash
./scripts/docker_run.sh
```

This command will firstly compile your code, and then start a local development network. You can also replace the default command (`cargo build --release && ./target/release/parrot --dev --ws-external`) by appending your own. A few useful ones are as follow.

```bash
# Run Substrate node without re-compiling 
./scripts/docker_run.sh ./target/release/parrot --dev --ws-external

# Purge the local dev chain
./scripts/docker_run.sh ./target/release/parrot purge-chain --dev

# Check whether the code is compilable
./scripts/docker_run.sh cargo check
```
