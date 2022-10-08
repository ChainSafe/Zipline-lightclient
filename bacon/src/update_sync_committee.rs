use super::types::*;
use super::utils::*;
use alloc::vec::Vec;
use alloc::string::String;


macro_rules! tryprintln {
    ($body:expr) => {
        // println!(body)
    };
}



pub fn process_sync_committee_period_update(
    prev_update: SyncCommitteePeriodUpdate,
    update: SyncCommitteePeriodUpdate,
    validators_root: H256,
) -> Result<(SyncCommittee, BeaconHeader), String> {
    let sync_committee_bits =
        get_sync_committee_bits(update.sync_aggregate.sync_committee_bits.clone())?;
    //     .map_err(|_| DispatchError::Other("Couldn't process sync committee bits"))?;
    tryprintln!("got sync committee bits");
    sync_committee_participation_is_supermajority(sync_committee_bits.clone())?;
    tryprintln!("sync committee participation is supermajority");
    verify_sync_committee(
        update.next_sync_committee.clone(),
        update.next_sync_committee_branch,
        update.finalized_header.state_root,
        NEXT_SYNC_COMMITTEE_DEPTH,
        NEXT_SYNC_COMMITTEE_INDEX,
    )?;
    tryprintln!("verified sync committee");
    let block_root: H256 = hash_tree_root_beacon_header(update.finalized_header.clone())?.into();
    verify_header(
        block_root,
        update.finality_branch,
        update.attested_header.state_root,
        FINALIZED_ROOT_DEPTH,
        FINALIZED_ROOT_INDEX,
    )?;

    // let current_period = compute_current_sync_period(update.attested_header.slot);
    // let current_sync_committee = Self::get_sync_committee_for_period(current_period)?;
    let current_sync_committee = prev_update.next_sync_committee;
    // let validators_root = <ValidatorsRoot<T>>::get();

    verify_signed_header(
        sync_committee_bits,
        update.sync_aggregate.sync_committee_signature,
        current_sync_committee.pubkeys,
        update.fork_version,
        update.attested_header,
        validators_root,
    )?;

    // Self::store_sync_committee(current_period + 1, update.next_sync_committee);
    // Self::store_finalized_header(block_root, update.finalized_header);
    tryprintln!("verified signed header");
    Ok((update.next_sync_committee, update.finalized_header))
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

    if is_valid_merkle_branch(
        sync_committee_root.into(),
        sync_committee_branch,
        depth,
        index,
        header_state_root,
    ) {
        return Ok(());
    } else {
        return Err("Sync committee merkle branch is invalid".into());
    }
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
