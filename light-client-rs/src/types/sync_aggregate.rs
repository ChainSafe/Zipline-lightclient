use crate::constants::*;

use alloc::vec;
use alloc::vec::Vec;
pub use ssz_rs::{
    prelude::Vector, Bitvector, Deserialize, SimpleSerialize as SimpleSerializeTrait, Sized,
};
use ssz_rs_derive::SimpleSerialize;

pub struct SyncAggregate {
    pub sync_committee_bits: Bitvector<SYNC_COMMITTEE_SIZE>,
    pub sync_committee_signature: Vec<u8>,
}

#[derive(Default, SimpleSerialize, Debug)]
pub struct SSZSyncAggregate {
    pub sync_committee_bits: Bitvector<SYNC_COMMITTEE_SIZE>,
    pub sync_committee_signature: Vector<u8, SIGNATURE_SIZE>,
}

impl From<SSZSyncAggregate> for SyncAggregate {
    fn from(value: SSZSyncAggregate) -> Self {
        SyncAggregate {
            sync_committee_bits: value.sync_committee_bits,
            sync_committee_signature: value.sync_committee_signature.to_vec(),
        }
    }
}
