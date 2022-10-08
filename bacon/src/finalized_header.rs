use super::types::*;
use super::utils::*;
use alloc::vec::Vec;
use alloc::string::String;


pub fn process_finalized_header(update: FinalizedHeaderUpdate, sync_committee:SyncCommittee, validators_root: H256) -> Result<BeaconHeader, String> {
    let sync_committee_bits = get_sync_committee_bits(update.sync_aggregate.sync_committee_bits.clone())?;
    sync_committee_participation_is_supermajority(sync_committee_bits.clone())?;

    let block_root = H256(hash_tree_root_beacon_header(update.finalized_header.clone())?);
        
    verify_header(
        block_root,
        update.finality_branch,
        update.attested_header.state_root,
        FINALIZED_ROOT_DEPTH,
        FINALIZED_ROOT_INDEX,
    )?;

    // let current_period = compute_current_sync_period(update.attested_header.slot);
    // let sync_committee = Self::get_sync_committee_for_period(current_period)?;

    verify_signed_header(
        sync_committee_bits,
        update.sync_aggregate.sync_committee_signature,
        sync_committee.pubkeys,
        update.fork_version,
        update.attested_header,
        validators_root,
    )?;

    // Self::store_finalized_header(block_root, update.finalized_header);

    Ok(update.finalized_header)
}

fn sync_committee_participation_is_supermajority(sync_committee_bits: Vec<u8>) -> Result<(), String> {
    let sync_committee_sum = get_sync_committee_sum(sync_committee_bits.clone());
    
    if sync_committee_sum * 3 >= sync_committee_bits.clone().len() as u64 * 2 {
        Ok(())
    } else {
        Err("Sync committee participation is not supermajority".into())
    }
}