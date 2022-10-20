use crate::constants::mainnet::*;
use crate::types::*;
use crate::utils::*;

use alloc::string::String;
use alloc::vec::Vec;
use ssz_rs::prelude::Vector;

/// Used to compared adjacent `SyncCommitteePeriodUpdate` messages
///
/// Returns `Ok(())` if the two updates are adjacent and consistent. If the prev_update was trusted then according to the
/// light client protocol it is not ok to trust the second update as well.
///
/// The validators_root uniquely identifies which chain these updates must belong to. Use `crate::constants::mainnet::VALIDATORS_ROOT`
/// for the Ethereum beacon chain mainnet
pub fn check_sync_committee_period_update(
    prev_update: SyncCommitteePeriodUpdate,
    update: SyncCommitteePeriodUpdate,
    validators_root: H256,
) -> Result<(), String> {
    let sync_committee_bits =
        get_sync_committee_bits(update.sync_aggregate.sync_committee_bits.clone())?;

    sync_committee_participation_is_supermajority(sync_committee_bits.clone())?;
    verify_sync_committee(
        update.next_sync_committee.clone(),
        update.next_sync_committee_branch,
        update.finalized_header.state_root,
        NEXT_SYNC_COMMITTEE_DEPTH,
        NEXT_SYNC_COMMITTEE_INDEX,
    )?;
    let block_root: H256 = hash_tree_root_beacon_header(update.finalized_header.clone())?.into();
    verify_header(
        block_root,
        update.finality_branch,
        update.attested_header.state_root,
        FINALIZED_ROOT_DEPTH,
        FINALIZED_ROOT_INDEX,
    )?;

    let current_sync_committee = prev_update.next_sync_committee;

    verify_signed_header(
        sync_committee_bits,
        update.sync_aggregate.sync_committee_signature,
        current_sync_committee.pubkeys,
        update.fork_version,
        update.attested_header,
        validators_root,
    )?;

    Ok(())
}

fn sync_committee_participation_is_supermajority(
    sync_committee_bits: Vec<u8>,
) -> Result<(), String> {
    let sync_committee_sum = get_sync_committee_sum(sync_committee_bits.clone());
    if sync_committee_sum * 3 >= sync_committee_bits.clone().len() as u64 * 2 {
        return Ok(());
    } else {
        return Err("Sync committee participation is not supermajority".into());
    }
}

fn verify_sync_committee(
    sync_committee: SyncCommittee,
    sync_committee_branch: Vec<H256>,
    header_state_root: H256,
    depth: u64,
    index: u64,
) -> Result<(), String> {
    let sync_committee_root = hash_tree_root_sync_committee(sync_committee)?;

    is_valid_merkle_branch(
        sync_committee_root.into(),
        sync_committee_branch,
        depth,
        index,
        header_state_root,
    )
}

fn hash_tree_root_sync_committee(sync_committee: SyncCommittee) -> Result<[u8; 32], String> {
    let mut pubkeys_vec = Vec::new();

    for pubkey in sync_committee.pubkeys.iter() {
        let conv_pubkey = Vector::<u8, 48>::from_iter(pubkey.0);

        pubkeys_vec.push(conv_pubkey);
    }

    let pubkeys = Vector::<Vector<u8, 48>, { SYNC_COMMITTEE_SIZE }>::from_iter(pubkeys_vec.clone());

    let agg = Vector::<u8, 48>::from_iter(sync_committee.aggregate_pubkey.0);

    hash_tree_root(SSZSyncCommittee {
        pubkeys: pubkeys,
        aggregate_pubkey: agg,
    })
}
