//! # Ethereum Beacon Client
// #![cfg_attr(not(feature = "std"), no_std)]
#![no_std]
extern crate alloc;


pub mod update_sync_committee;
pub mod types;
pub use types::*;
pub use update_sync_committee::*;
// use alloc::string::String;
pub use milagro_bls::{AggregatePublicKey, AggregateSignature, AmclError, Signature};
use ssz_rs::deserialize;
// pub use snowbridge_ethereum::H256;
pub use ssz_rs::{
    prelude::Vector, Bitvector, Deserialize, SimpleSerialize as SimpleSerializeTrait, Sized,
};

use alloc::vec::Vec;
use alloc::string::String;
use crate::alloc::string::ToString;




macro_rules! tryprintln {
    ($body:expr) => {
        // println!(body)
    };
}





pub fn ssz_process_sync_committee_period_update(
    prev_update: Vec<u8>,
    update: Vec<u8>,
    validators_root: H256,
) -> Result<(SyncCommittee, BeaconHeader), String> {
    tryprintln!("entry point");
    let prev_update: SSZSyncCommitteePeriodUpdate =
        deserialize(&prev_update).map_err(|_e| "Failed to decode previous update".to_string())?;
    tryprintln!("decode 1");
    let update: SSZSyncCommitteePeriodUpdate = SSZSyncCommitteePeriodUpdate::deserialize(&update)
        .map_err(|_| "Failed to decode update")?;
    tryprintln!("decode 2");

    let prev_update = SyncCommitteePeriodUpdate::from(prev_update);
    let update = SyncCommitteePeriodUpdate::from(update);

    process_sync_committee_period_update(prev_update, update, validators_root)
        .map_err(|_e| "failed sync comitte update period: {}".to_string())
}

