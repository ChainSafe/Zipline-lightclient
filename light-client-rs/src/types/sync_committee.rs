use crate::types::PublicKey;
pub use ssz_rs::{
    prelude::Vector, Bitvector, Deserialize, SimpleSerialize as SimpleSerializeTrait, Sized,
};
use crate::constants::*;

use ssz_rs_derive::SimpleSerialize;
use alloc::vec;
use alloc::vec::Vec;

#[derive(Clone, Debug)]
pub struct SyncCommittee {
    pub pubkeys: Vec<PublicKey>,
    pub aggregate_pubkey: PublicKey,
}

#[derive(Default, SimpleSerialize, Debug)]
pub struct SSZSyncCommittee {
    pub pubkeys: Vector<Vector<u8, PUBKEY_SIZE>, SYNC_COMMITTEE_SIZE>,
    pub aggregate_pubkey: Vector<u8, PUBKEY_SIZE>,
}

impl From<SSZSyncCommittee> for SyncCommittee {
    fn from(value: SSZSyncCommittee) -> Self {
        SyncCommittee {
            pubkeys: value
                .pubkeys
                .iter()
                .map(|pk| PublicKey(pk[..48].try_into().unwrap()))
                .collect(),
            aggregate_pubkey: PublicKey(value.aggregate_pubkey[..48].try_into().unwrap()),
        }
    }
}