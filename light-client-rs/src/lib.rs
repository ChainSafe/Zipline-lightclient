//! # Ethereum Beacon Client
#![no_std]
extern crate alloc;

mod types;
// mod ssz_types;
mod check_sync_committee_update;
mod utils;
mod constants;

pub use check_sync_committee_update::check_sync_committee_period_update;
pub use types::{H256, SyncCommitteePeriodUpdate};
