//! # Ethereum Beacon Client
// #![cfg_attr(not(feature = "std"), no_std)]
#![no_std]
extern crate alloc;

pub mod finalized_header;
pub mod types;
pub mod update_sync_committee;
pub mod utils;

pub use finalized_header::process_finalized_header;
pub use milagro_bls::{AggregatePublicKey, AggregateSignature, AmclError, Signature};
use ssz_rs::deserialize;
pub use ssz_rs::{
    prelude::Vector, Bitvector, Deserialize, SimpleSerialize as SimpleSerializeTrait, Sized,
};
pub use types::*;
pub use update_sync_committee::process_sync_committee_period_update;

use crate::alloc::string::ToString;
use alloc::string::String;

macro_rules! tryprintln {
    ($body:expr) => {
        // println!(body)
    };
}

pub fn ssz_process_sync_committee_period_update(
    prev_update: &[u8],
    update: &[u8],
    validators_root: H256,
) -> Result<(SyncCommittee, BeaconHeader), String> {
    // deserialize from bytes into structured types for the sync committee
    let prev_update = SyncCommitteePeriodUpdate::try_from(prev_update)?;
    let update = SyncCommitteePeriodUpdate::try_from(update)?;

    // Process the update between the prev and current updates
    // If it validates successfully returns Ok()
    // Otherwise returns the error string
    process_sync_committee_period_update(prev_update, update, validators_root)
}

pub fn ssz_process_finalized_header(
    update: &[u8],
    sync_committee: &[u8],
    validators_root: H256,
) -> Result<BeaconHeader, String> {
    tryprintln!("entry point");
    let update: SSZFinalizedHeaderUpdate =
        deserialize(&update).map_err(|_e| "Failed to decode previous update".to_string())?;
    tryprintln!("decode 1");
    let sync_committee: SSZSyncCommittee =
        SSZSyncCommittee::deserialize(&sync_committee).map_err(|_| "Failed to decode update")?;
    tryprintln!("decode 2");

    let update = FinalizedHeaderUpdate::from(update);
    let sync_committee = SyncCommittee::from(sync_committee);

    process_finalized_header(update, sync_committee, validators_root)
        .map_err(|_e| "failed sync comitte update period: {}".to_string())
}
