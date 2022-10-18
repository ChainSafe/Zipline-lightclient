use ssz_rs::Deserialize;
use ssz_rs::Sized;

mod beacon_block_header;
mod h256;
mod sync_aggregate;
mod sync_committee;
mod sync_committee_period_update;

pub use beacon_block_header::{BeaconBlockHeader, SSZBeaconBlockHeader};
pub use h256::H256;
pub use sync_aggregate::{SSZSyncAggregate, SyncAggregate};
pub use sync_committee::{SSZSyncCommittee, SyncCommittee};
pub use sync_committee_period_update::{SSZSyncCommitteePeriodUpdate, SyncCommitteePeriodUpdate};

pub type ForkVersion = [u8; 4];

pub type Domain = H256;
pub type Root = H256;

#[derive(Clone, PartialEq, Debug)]
pub struct PublicKey(pub [u8; 48]);

#[derive(Clone, Default, PartialEq)]
pub struct ForkData {
    // 1 or 0 bit, indicates whether a sync committee participated in a vote
    pub current_version: [u8; 4],
    pub genesis_validators_root: [u8; 32],
}

#[derive(Clone, Default, PartialEq)]
pub struct SigningData {
    pub object_root: Root,
    pub domain: Domain,
}

use alloc::vec;
use alloc::vec::Vec;
use ssz_rs_derive::SimpleSerialize;

#[derive(Default, SimpleSerialize, Debug)]
pub struct SSZForkData {
    pub current_version: [u8; 4],
    pub genesis_validators_root: [u8; 32],
}
#[derive(Default, SimpleSerialize, Debug)]
pub struct SSZSigningData {
    pub object_root: [u8; 32],
    pub domain: [u8; 32],
}
