# Demo Instructions

## Relayer

### Retrieve sync period updates from Lodestar

The following will call into a Eth2 client and get the state sync proofs for the 
current and previous sync periods. These are returned as SSZ serialized blobs

```shell
cd shortbarrel
yarn
tsc && node dist/index.js
```

This will write the serialized sync update messages in binary form to the `preimage-cache` directory

### Run off-chain validation to ensure the Eth2 node was being honest

```shell
cd shortcut-ts
cargo run 
```