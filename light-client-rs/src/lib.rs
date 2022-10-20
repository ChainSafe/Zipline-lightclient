#![no_std]
#![doc = include_str!("../README.md")]

extern crate alloc;

mod check_sync_committee_update;
pub mod constants;
mod types;
mod utils;

pub use check_sync_committee_update::check_sync_committee_period_update;
pub use types::{
    BeaconBlockHeader, PublicKey, SyncAggregate, SyncCommittee, SyncCommitteePeriodUpdate, H256,
};
