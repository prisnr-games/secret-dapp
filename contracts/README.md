# Contracts for Secret Prisoners

## Install Rust and set up build environment for contract compilation (one time only)

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

rustup default stable
rustup target list --installed
rustup target add wasm32-unknown-unknown

rustup install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly

apt install build-essential

cargo install cargo-generate --features vendored-openssl
```

## Compile optimized version of the secret prisoner contract

```sh
cd secret-prisoner-game-contract

docker run --rm -v "$(pwd)":/contract \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  enigmampc/secret-contract-optimizer:1.0.5
```

To get the code hash for the compiled contract run this:

```sh
gunzip -c contract.wasm.gz > contract.wasm
shasum -a 256 contract.wasm
```

## Setting up secretdev local testnet chain

```sh
docker run -it --rm \
 -p 26657:26657 -p 26656:26656 -p 1317:1317 \
 -v $(pwd):/root/code \
 --name secretdev enigmampc/secret-network-sw-dev:v1.2.0
```

### Uploading contract to local dev chain

In a new terminal window connect to the testnet container:

```sh
docker exec -it secretdev /bin/bash
```



