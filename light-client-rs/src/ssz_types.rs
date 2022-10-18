use alloc::vec;
use alloc::vec::Vec;

use crate::constants::*;
use crate::types::ForkVersion;

pub use milagro_bls::{AggregatePublicKey, AggregateSignature, AmclError, Signature};
pub use ssz_rs::{
    prelude::Vector, Bitvector, Deserialize, SimpleSerialize as SimpleSerializeTrait, Sized,
};
use ssz_rs_derive::SimpleSerialize;

#[derive(Default, SimpleSerialize, Debug)]
pub struct SSZSyncCommittee {
    pub pubkeys: Vector<Vector<u8, PUBKEY_SIZE>, SYNC_COMMITTEE_SIZE>,
    pub aggregate_pubkey: Vector<u8, PUBKEY_SIZE>,
}

#[derive(Default, SimpleSerialize, Debug)]
pub struct SSZBeaconBlockHeader {
    pub slot: u64,
    pub proposer_index: u64,
    pub parent_root: [u8; 32],
    pub state_root: [u8; 32],
    pub body_root: [u8; 32],
}
#[derive(Default, SimpleSerialize, Debug)]
pub struct SSZSyncAggregate {
    pub sync_committee_bits: Bitvector<SYNC_COMMITTEE_SIZE>,
    pub sync_committee_signature: Vector<u8, SIGNATURE_SIZE>,
}

#[derive(Default, SimpleSerialize, Debug)]
pub struct SSZSyncCommitteePeriodUpdate {
    pub attested_header: SSZBeaconBlockHeader,
    pub next_sync_committee: SSZSyncCommittee,
    pub next_sync_committee_branch: Vector<[u8; 32], 5>,
    pub finalized_header: SSZBeaconBlockHeader,
    pub finality_branch: Vector<[u8; 32], 6>,
    pub sync_aggregate: SSZSyncAggregate,
    pub fork_version: ForkVersion,
}
