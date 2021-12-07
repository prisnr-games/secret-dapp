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

## Compile local version of the secret prisoner contract

```sh
cd secret-prisoner-game-contract

make build
```

This version includes `debug-print` statements.

## Compile reproducible mainnet version of the secret prisoner contract

```sh
cd secret-prisoner-game-contract

make build-mainnet-reproducible
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
 --name secretdev enigmampc/secret-network-sw-dev:v1.2.2-1
```

If you are not running this command from the `contracts` directory, adjust the `$(pwd)` part of the `-v` parameter to mount the contract directory in the container.

## Uploading contracts to local dev chain

In a new terminal window connect to the testnet container:

```sh
docker exec -it secretdev /bin/bash
```

### Setting up Minter contract

Now let's create the minter contract for secret prisoner powerup nfts.

```sh
cd code/secret-prisoner-minter
secretd tx compute store contract.wasm.gz --from a --gas 4500000 -y --keyring-backend test
```

We initialize the minter contract

```sh
MINTER_CODE_ID=1

MINTER_INIT='{"name": "secret-prisoner-powerup-nft-minter", "symbol": "sprispowrup", "entropy": "secret stuff for minter"}'

secretd tx compute instantiate $MINTER_CODE_ID "$MINTER_INIT" --from a --label "secret-prisoners-minter-0.0.1" -y --keyring-backend test --gas 35000
```

Query the transaction hash to get the minter contract's address. On the local dev network the second uploaded contract should have the following address:

```sh
MINTER_CONTRACT=secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg
MINTER_CODE_HASH=36d94d2066b903ede9716f77f2fe99274aaa2c434feb424302dbb8aef9f34721
```

To load the contract in the container shell enter:

```sh
cd ../secret-prisoner-game-contract
secretd tx compute store contract.wasm.gz --from a --gas 3000000 -y --keyring-backend test
```

You can confirm that the contract was uploaded by querying the transaction hash:

```sh
secretd q tx {txhash}
```

Now we initialize the contract from the test user `a`, and seed the jackpot pool with 10 SCRT. The `CODE_ID` might be different if you've uploaded other contracts. Change `INIT` if you do not want the colors and shapes to have equal probability:

```sh
CODE_ID=2

INIT='{"rounds_per_game": 1, "stakes": "1000000", "entropy": "secret stuff", "red_weight": 25, "green_weight": 25, "blue_weight": 25, "black_weight": 25, "triangle_weight": 25, "square_weight": 25, "circle_weight": 25, "star_weight": 25, "minter": {"code_hash":"36d94d2066b903ede9716f77f2fe99274aaa2c434feb424302dbb8aef9f34721", "address":"secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg"}}'

secretd tx compute instantiate $CODE_ID "$INIT" --from a --label "secret-prisoners-0.0.1" -y --keyring-backend test --amount 10000000uscrt --gas 70000
```

You can query the transaction hash to make sure that the contract was initialized and get the contract address:

```sh
secretd q tx {txhash}
```

On the local dev network the second uploaded contract should have the following address:

```sh
CONTRACT=secret10pyejy66429refv3g35g2t7am0was7ya6hvrzf
CODE_HASH=4e1962b82a2bac958c1bb92fe7283cea94dce967d74d267eaa2ce3b43b9b43cf
```

### Set game contract as a minter

```sh
secretd tx compute execute $MINTER_CONTRACT '{"set_minters": {"minters": ["secret10pyejy66429refv3g35g2t7am0was7ya6hvrzf"]}}' --from a --keyring-backend test --gas 28000 -y
```

## Command line interaction with the contract

Each player can interact with the player by sending `join`, `submit`, `guess`, and `pick_reward` messages to the contract. Only some messages are valid depending on the state of the game.

### Joining a game

Player `a` joining a game, sending 1 SCRT wager.

```sh
secretd tx compute execute $CONTRACT '{"join":{}}' --from a --keyring-backend test --gas 35000 --amount 1000000uscrt -y
```

Player `b` joining a game, sending 1 SCRT wager.

```sh
secretd tx compute execute $CONTRACT '{"join":{}}' --from b --keyring-backend test --gas 35000 --amount 1000000uscrt -y
```

### Submitting hints to other player

Player `a` submits first hint to player `b`.

```sh
secretd tx compute execute $CONTRACT '{"submit":{"target":"i_have","color":"red"}}' --from a --keyring-backend test --gas 40000 -y
```

Player `b` submits first hint to player `a`.

```sh
secretd tx compute execute $CONTRACT '{"submit":{"target":"nobody_has","shape":"triangle"}}' --from b --keyring-backend test --gas 40000 -y
```

Player `a` submits second hint to player `b`.

```sh
secretd tx compute execute $CONTRACT '{"submit":{"target":"nobody_has","color":"black"}}' --from a --keyring-backend test --gas 40000 -y
```

Player `b` submits second hint to player `a`.

```sh
secretd tx compute execute $CONTRACT '{"submit":{"target":"i_have","shape":"star"}}' --from b --keyring-backend test --gas 40000 -y
```

### Guessing answer

Player `a` guesses bag is green triangle.

```sh
secretd tx compute execute $CONTRACT '{"guess":{"target":"bag","shape":"triangle","color":"green"}}' --from a --keyring-backend test --gas 40000 -y
```

Player `b` guesses player `a` has a blue circle.

```sh
secretd tx compute execute $CONTRACT '{"guess":{"target":"opponent","shape":"circle","color":"blue"}}' --from b --keyring-backend test --gas 40000 -y
```

### Picking reward

Player `a` picks jackpot from the pool.

```sh
secretd tx compute execute $CONTRACT '{"pick_reward": {"reward": "pool"}}' --from a --keyring-backend test --gas 100000 -y
```

Player `b` picks nft.

```sh
secretd tx compute execute $CONTRACT '{"pick_reward": {"reward": "nft"}}' --from b --keyring-backend test --gas 100000 -y
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
                    "secret10pyejy66429refv3g35g2t7am0was7ya6hvrzf",
                    "secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg"
                ],
                "permissions": ["owner"]
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

secretd tx sign-doc ./permit.json --from a > ./sig-a.json
```

To execute a game status query with permit for player a:

```sh
secretd q compute query $CONTRACT '{"with_permit":{"query":{"game_state":{}},"permit":{"params":{"permit_name":"Scrt Prisoners","allowed_tokens":["secret10pyejy66429refv3g35g2t7am0was7ya6hvrzf","secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg"],"chain_id":"secretdev-1","permissions":["owner"]},"signature":'"$(cat ./sig-a.json)"'}}}'
```

Repeat the same for player b replacing `--from a` with `--from b` and `sig-a.json` with `sig-b.json`.

### Querying for tokens that player a owns in minter

```sh
secretd q compute query $MINTER_CONTRACT '{"with_permit":{"query":{"tokens":{"owner":"secret..."}},"permit":{"permit_name":"Scrt Prisoners","allowed_tokens":["secret10pyejy66429refv3g35g2t7am0was7ya6hvrzf","secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg"],"chain_id":"secretdev-1","permissions":["owner"]},"signature":'"$(cat ./sig-a.json)"'}}}'
```

### Querying private metadata of a token owned by player a

```sh
secretd q compute query $MINTER_CONTRACT '{"with_permit":{"query":{"private_metadata":{"token_id":"secret..."}},"permit":{"permit_name":"Scrt Prisoners","allowed_tokens":["secret10pyejy66429refv3g35g2t7am0was7ya6hvrzf","secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg"],"chain_id":"secretdev-1","permissions":["owner"]},"signature":'"$(cat ./sig-a.json)"'}}}'
```