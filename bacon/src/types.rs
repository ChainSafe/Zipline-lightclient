use alloc::vec;
use alloc::vec::Vec;
use sha2::{Digest, Sha256};
use alloc::string::String;

pub use milagro_bls::{AggregatePublicKey, AggregateSignature, AmclError, Signature};
// pub use snowbridge_ethereum::H256;
pub use ssz_rs::{
    prelude::Vector, Bitvector, Deserialize, SimpleSerialize as SimpleSerializeTrait, Sized,
};
use ssz_rs_derive::SimpleSerialize;
macro_rules! tryprintln {
    ($body:expr) => {
        // tryprintln!(body)
    };
}


// use alloc::vec;
// use alloc::vec::Vec;
// use alloc::format;
pub type ForkVersion = [u8; 4];
pub const SYNC_COMMITTEE_SIZE: usize = 512;
pub const PUBKEY_SIZE: usize = 48;
pub const SIGNATURE_SIZE: usize = 96;
pub const NEXT_SYNC_COMMITTEE_DEPTH: u64 = 5;
pub const NEXT_SYNC_COMMITTEE_INDEX: u64 = 23;
pub const FINALIZED_ROOT_DEPTH: u64 = 6;
pub const FINALIZED_ROOT_INDEX: u64 = 41;
pub const DOMAIN_SYNC_COMMITTEE: [u8; 4] = [7, 0, 0, 0];
pub const GENESIS_FORK_VERSION: ForkVersion = [30, 30, 30, 30];

pub const SLOTS_PER_EPOCH: u64 = 32;
pub const EPOCHS_PER_SYNC_COMMITTEE_PERIOD: u64 = 256;
pub const IS_MINIMAL: bool = false;

pub type Domain = H256;
pub type Root = H256;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct H256(pub [u8; 32]);

pub(super) fn sha2_256(data: &[u8]) -> H256 {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut h = H256::default();
    h.0.copy_from_slice(&result);
    h
}

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

#[derive(Clone, Debug)]
pub struct BeaconHeader {
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

pub struct SyncAggregate {
    // both of these were bounded vecs
    // #[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
    pub sync_committee_bits: Bitvector<SYNC_COMMITTEE_SIZE>,
    // #[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
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
#[derive(Default, SimpleSerialize)]
pub struct SSZSyncCommittee {
    pub pubkeys: Vector<Vector<u8, PUBKEY_SIZE>, SYNC_COMMITTEE_SIZE>,
    pub aggregate_pubkey: Vector<u8, PUBKEY_SIZE>,
}
#[derive(Default, SimpleSerialize, Clone, Debug)]
pub struct SSZBeaconBlockHeader {
    pub slot: u64,
    pub proposer_index: u64,
    pub parent_root: [u8; 32],
    pub state_root: [u8; 32],
    pub body_root: [u8; 32],
}
#[derive(Default, Debug, SimpleSerialize, Clone)]
pub struct SSZSyncAggregate {
    pub sync_committee_bits: Bitvector<SYNC_COMMITTEE_SIZE>,
    pub sync_committee_signature: Vector<u8, SIGNATURE_SIZE>,
}

#[derive(Default, SimpleSerialize)]
pub struct SSZForkData {
    pub current_version: [u8; 4],
    pub genesis_validators_root: [u8; 32],
}
#[derive(Default, SimpleSerialize)]
pub struct SSZSigningData {
    pub object_root: [u8; 32],
    pub domain: [u8; 32],
}

#[derive(Default, SimpleSerialize)]
pub struct SSZSyncCommitteePeriodUpdate {
    pub attested_header: SSZBeaconBlockHeader,
    pub next_sync_committee: SSZSyncCommittee,
    // was a bounded vec
    pub next_sync_committee_branch: Vector<[u8; 32], 5>,
    pub finalized_header: SSZBeaconBlockHeader,
    // was a bounded vec
    pub finality_branch: Vector<[u8; 32], 6>,
    pub sync_aggregate: SSZSyncAggregate,
    // #[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_fork_version"))]
    pub fork_version: ForkVersion,
}

#[derive(Clone, Debug)]
pub struct SyncCommittee {
    // should this be a smallvec???
    pub pubkeys: Vec<PublicKey>,
    pub aggregate_pubkey: PublicKey,
}
pub struct SyncCommitteePeriodUpdate {
    pub attested_header: BeaconHeader,
    pub next_sync_committee: SyncCommittee,
    // was a bounded vec
    pub next_sync_committee_branch: Vec<H256>,
    pub finalized_header: BeaconHeader,
    // was a bounded vec
    pub finality_branch: Vec<H256>,
    pub sync_aggregate: SyncAggregate,
    // #[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_fork_version"))]
    pub fork_version: ForkVersion,
}

impl From<SSZSyncCommitteePeriodUpdate> for SyncCommitteePeriodUpdate {
    fn from(value: SSZSyncCommitteePeriodUpdate) -> Self {
        tryprintln!("from ssz sync committee period update");
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

impl From<SSZBeaconBlockHeader> for BeaconHeader {
    fn from(value: SSZBeaconBlockHeader) -> Self {
        tryprintln!("from ssz beacon block header");
        BeaconHeader {
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
        tryprintln!("from ssz sync committee");
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
        tryprintln!("from ssz sync aggregate");
        SyncAggregate {
            sync_committee_bits: value.sync_committee_bits,
            sync_committee_signature: value.sync_committee_signature.to_vec(),
        }
    }
}