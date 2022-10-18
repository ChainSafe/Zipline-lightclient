use ssz_rs::deserialize;
use crate::alloc::string::ToString;

use alloc::vec::Vec;
use alloc::string::String;

pub use milagro_bls::{AggregatePublicKey, AggregateSignature, AmclError, Signature};
pub use ssz_rs::{
    prelude::Vector, Bitvector, Deserialize, SimpleSerialize as SimpleSerializeTrait, Sized,
};

use crate::constants::*;
use crate::ssz_types::*;

pub type ForkVersion = [u8; 4];

pub type Domain = H256;
pub type Root = H256;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct H256(pub [u8; 32]);

impl From<[u8; 32]> for H256 {
    fn from(bytes: [u8; 32]) -> Self {
        H256(bytes)
    }
}

impl From<H256> for [u8; 32] {
    fn from(h: H256) -> Self {
        h.0
    }
}

impl H256 {
    pub fn from_slice(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != 32 {
            return Err("Invalid length for H256".into());
        }
        let mut h = H256::default();
        h.0.copy_from_slice(bytes);
        Ok(h)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct PublicKey(pub [u8; 48]);

pub struct SyncCommitteePeriodUpdate {
    pub attested_header: BeaconBlockHeader,
    pub next_sync_committee: SyncCommittee,
    pub next_sync_committee_branch: Vec<H256>,
    pub finalized_header: BeaconBlockHeader,
    pub finality_branch: Vec<H256>,
    pub sync_aggregate: SyncAggregate,
    pub fork_version: ForkVersion,
}

#[derive(Clone, Debug)]
pub struct BeaconBlockHeader {
    // The slot for which this block is created. Must be greater than the slot of the block defined
    // by parentRoot.
    pub slot: u64,
    // The index of the validator that proposed the block.
    pub proposer_index: u64,
    // The block root of the parent block, forming a block chain.
    pub parent_root: Root,
    // The hash root of the post state of running the state transition through this block.
    pub state_root: Root,
    // The hash root of the beacon block body
    pub body_root: Root,
}

#[derive(Clone, Debug)]
pub struct SyncCommittee {
    pub pubkeys: Vec<PublicKey>,
    pub aggregate_pubkey: PublicKey,
}

pub struct SyncAggregate {
    pub sync_committee_bits: Bitvector<SYNC_COMMITTEE_SIZE>,
    pub sync_committee_signature: Vec<u8>,
}

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


impl TryFrom<&[u8]> for SyncCommitteePeriodUpdate {
    type Error = String;
    fn try_from(bytes: &[u8]) -> Result<Self, String> {
        let ssz_form: SSZSyncCommitteePeriodUpdate =
            deserialize(&bytes).map_err(|_e| "Failed to decode previous update".to_string())?;
        Ok(Self::from(ssz_form))
    }
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

impl From<SSZBeaconBlockHeader> for BeaconBlockHeader {
    fn from(value: SSZBeaconBlockHeader) -> Self {
        BeaconBlockHeader {
            slot: value.slot,
            proposer_index: value.proposer_index,
            parent_root: value.parent_root.into(),
            state_root: value.state_root.into(),
            body_root: value.body_root.into(),
        }
    }
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

impl From<SSZSyncAggregate> for SyncAggregate {
    fn from(value: SSZSyncAggregate) -> Self {
        SyncAggregate {
            sync_committee_bits: value.sync_committee_bits,
            sync_committee_signature: value.sync_committee_signature.to_vec(),
        }
    }
}