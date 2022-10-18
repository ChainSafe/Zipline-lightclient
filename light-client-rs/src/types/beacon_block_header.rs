use ssz_rs::Sized;
use ssz_rs::Deserialize;
use alloc::string::String;
use crate::types::Root;

use ssz_rs_derive::SimpleSerialize;
use alloc::vec;
use alloc::vec::Vec;

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

#[derive(Default, SimpleSerialize, Debug)]
pub struct SSZBeaconBlockHeader {
    pub slot: u64,
    pub proposer_index: u64,
    pub parent_root: [u8; 32],
    pub state_root: [u8; 32],
    pub body_root: [u8; 32],
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

pub fn get_ssz_beacon_header(beacon_header: BeaconBlockHeader) -> Result<SSZBeaconBlockHeader, String> {
    Ok(SSZBeaconBlockHeader {
        slot: beacon_header.slot,
        proposer_index: beacon_header.proposer_index,
        parent_root: beacon_header
            .parent_root
            .as_bytes()
            .try_into()
            .map_err(|_| "MerkleizationError::InvalidLength")?,
        state_root: beacon_header
            .state_root
            .as_bytes()
            .try_into()
            .map_err(|_| "MerkleizationError::InvalidLength")?,
        body_root: beacon_header
            .body_root
            .as_bytes()
            .try_into()
            .map_err(|_| "MerkleizationError::InvalidLength")?,
    })
}
