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
 --name secretdev enigmampc/secret-network-sw-dev:v1.2.0-1
```

If you are not running this command from the contract directory, adjust the `$(pwd)` part of the `-v` parameter to mount the contract directory in the container.

### Uploading contract to local dev chain

In a new terminal window connect to the testnet container:

```sh
docker exec -it secretdev /bin/bash
```

To load the contract in the container shell enter:

```sh
cd code/
secretd tx compute store contract.wasm.gz --from a --gas 2500000 -y --keyring-backend test
```

You can confirm that the contract was uploaded by querying the transaction hash:

```sh
secretd q tx {txhash}
```

Now we initialize the contract from the test user `a`. The `CODE_ID` might be different if you've uploaded other contracts. Change `INIT` if you do not want the colors and shapes to have equal probability:

```sh
CODE_ID=1

INIT='{"rounds_per_game": 1, "red_weight": 25, "green_weight": 25, "blue_weight": 25, "black_weight": 25, "triangle_weight": 25, "square_weight": 25, "circle_weight": 25, "star_weight": 25}'

secretd tx compute instantiate $CODE_ID "$INIT" --from a --label "secret-prisoners-0.0.1" -y --keyring-backend test --gas 30000
```

You can query the transaction hash to make sure that the contract was initialized and get the contract address:

```sh
secretd q tx {txhash}
```

On the local dev network the first uploaded contract should have the following address:

```sh
CONTRACT=secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg
```

## Command line interaction with the contract

### Joining a game

Player `a` joining a game.

```sh
secretd tx compute execute $CONTRACT '{"join":{}}' --from a --keyring-backend test --gas 32000 -y
```

Player `b` joining a game.

```sh
secretd tx compute execute $CONTRACT '{"join":{}}' --from b --keyring-backend test --gas 32000 -y
```

### Creating a query permit

In order to create a query permit for test user `a` on the command line do the following (modify `allowed_tokens` to have the contract's address as needed):

```sh
echo '{
    "chain_id": "secretdev-1",
    "account_number": "0",
    "sequence": "0",
    "msgs": [
        {
            "type": "query_permit",
            "value": {
                "permit_name": "Scrt Prisoners",
                "allowed_tokens": [
                    "secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg"
                ],
                "permissions": ["owner"],
            }
        }
    ],
    "fee": {
        "amount": [
            {
                "denom": "uscrt",
                "amount": "0"
            }
        ],
        "gas": "1"
    },
    "memo": ""
}' > ./permit.json

secretd tx sign-doc ./permit.json --from a > ./sig.json

```