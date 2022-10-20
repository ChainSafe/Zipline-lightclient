# light-client-rs

A crate for working with Ethereum beacon chain light client protocol messages. `no_std` friendly!

!! For hacking and experimentation only. This is NOT production ready code !!

This can be used to deserialize and inspect SSZ encoded `SyncCommitteePeriodUpdate` messages that are retrieved from a full node RPC. It can also be used to verify a transition between two adjacent sync period updates is valid using `check_sync_committee_period_update`.

## Usage

SSZ encoded `SyncCommitteePeriodUpdate`s can be deserialized using the `try_from` with a byte slice.

```rust
use eth_lightclient::SyncCommitteePeriodUpdate;

const BYTES: &[u8] = include_bytes!(
    "../tests/sync-updates/0xe4c2cee3a9455c2b7c0449152a8c7e1a7b811353e4ea2c1dbe1cbe0c790b45f7"
);

let update = SyncCommitteePeriodUpdate::try_from(BYTES);
```

Adjacent period updates can be verified given a chain validators root. This checks that:

- The second update has enough signatures
- The signatures are from the committee defined in the prior update
- The next sync committee proof is valid with respect to the finalized block root
- .. some other stuff

From a trusted finalized block root and committee in one sync period this allows proving that a new committee and finalized block root in the next sync period can also be trusted. This allows a light client to jump to a new trusted block root every sync period.

```rust
use eth_lightclient::{
    SyncCommitteePeriodUpdate, H256,
    check_sync_committee_period_update,
    constants::mainnet::VALIDATORS_ROOT,
};

const A: &[u8] = include_bytes!(
    "../tests/sync-updates/0xe4c2cee3a9455c2b7c0449152a8c7e1a7b811353e4ea2c1dbe1cbe0c790b45f7"
);
const B: &[u8] = include_bytes!(
    "../tests/sync-updates/0x78ae69239826edd5ac0abfe3a69e916e7479ad44e834e35a08e4df7601732a85"
);

check_sync_committee_period_update(
    SyncCommitteePeriodUpdate::try_from(A).unwrap(),
    SyncCommitteePeriodUpdate::try_from(B).unwrap(),
    H256(VALIDATORS_ROOT),
) // returns Ok(())
```

## Authors

- Willem Olding (ChainSafe Systems)
- Eric Tu (ChainSafe Systems)
- Cayman Nava (ChainSafe Systems)

## Acknowledgements

This crate was developed as part of the EthBogota hackathon for the project [Zipline](https://github.com/willemolding/zipline). Zipline is a trustless Eth beacon chain to EVM bridge that uses the beacon chain protocol.

Credit to the original authors of the sync period verification - [Snowfork](https://github.com/Snowfork/snowbridge). Their original implementation was tightly coupled to the Substrate ecosystem and not useable outside of that context hence the refactor.
