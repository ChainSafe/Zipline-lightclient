use crate::types::*;

use crate::alloc::string::ToString;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
pub use ssz_rs::{
    deserialize, prelude::Vector, Bitvector, Deserialize, SimpleSerialize as SimpleSerializeTrait,
    Sized,
};
use ssz_rs_derive::SimpleSerialize;

pub struct SyncCommitteePeriodUpdate {
    pub attested_header: BeaconBlockHeader,
    pub next_sync_committee: SyncCommittee,
    pub next_sync_committee_branch: Vec<H256>,
    pub finalized_header: BeaconBlockHeader,
    pub finality_branch: Vec<H256>,
    pub sync_aggregate: SyncAggregate,
    pub fork_version: ForkVersion,
}

impl TryFrom<&[u8]> for SyncCommitteePeriodUpdate {
    type Error = String;
    fn try_from(bytes: &[u8]) -> Result<Self, String> {
        let ssz_form: SSZSyncCommitteePeriodUpdate =
            deserialize(&bytes).map_err(|_e| "Failed to decode previous update".to_string())?;
        Ok(Self::from(ssz_form))
    }
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

impl From<SSZSyncCommitteePeriodUpdate> for SyncCommitteePeriodUpdate {
    fn from(value: SSZSyncCommitteePeriodUpdate) -> Self {
        SyncCommitteePeriodUpdate {
            attested_header: value.attested_header.into(),
            next_sync_committee: value.next_sync_committee.into(),
            next_sync_committee_branch: value
                .next_sync_committee_branch
                .iter()
                .map(|v| H256(*v))
                .collect(),
            finalized_header: value.finalized_header.into(),
            finality_branch: value.finality_branch.iter().map(|v| H256(*v)).collect(),
            sync_aggregate: value.sync_aggregate.into(),
            fork_version: value.fork_version,
        }
    }
}
