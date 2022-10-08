//! # Ethereum Beacon Client
// #![cfg_attr(not(feature = "std"), no_std)]
#![no_std]
extern crate alloc;
use alloc::string::String;
pub use milagro_bls::{AggregatePublicKey, AggregateSignature, AmclError, Signature};
use ssz_rs::deserialize;
// pub use snowbridge_ethereum::H256;
pub use ssz_rs::{
    prelude::Vector, Bitvector, Deserialize, SimpleSerialize as SimpleSerializeTrait, Sized,
};
use ssz_rs_derive::SimpleSerialize;

use alloc::vec;
use alloc::vec::Vec;
use alloc::format;
use sha2::{Digest, Sha256};

pub type ForkVersion = [u8; 4];
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
pub const SYNC_COMMITTEE_SIZE: usize = 512;
pub const IS_MINIMAL: bool = false;

pub type Domain = H256;
pub type Root = H256;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct H256(pub [u8; 32]);

fn sha2_256(data: &[u8]) -> H256 {
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

#[derive(Clone, PartialEq)]
pub struct PublicKey(pub [u8; 48]);

#[derive(Clone)]
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
    pub sync_committee_bits: Vec<u8>,
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
    pub sync_committee_period: u64,
}

#[derive(Clone)]
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
    pub sync_committee_period: u64,
}

impl From<SSZSyncCommitteePeriodUpdate> for SyncCommitteePeriodUpdate {
    fn from(value: SSZSyncCommitteePeriodUpdate) -> Self {
        SyncCommitteePeriodUpdate {
            attested_header: value.attested_header.into(),
            next_sync_committee: value.next_sync_committee.into(),
            next_sync_committee_branch: value.next_sync_committee_branch.iter().map(|v| H256(*v)).collect(),
            finalized_header: value.finalized_header.into(),
            finality_branch: value.finality_branch.iter().map(|v| H256(*v)).collect(),
            sync_aggregate: value.sync_aggregate.into(),
            fork_version: value.fork_version,
            sync_committee_period: value.sync_committee_period,
        }
    }
}

impl From<SSZBeaconBlockHeader> for BeaconHeader {
    fn from(value: SSZBeaconBlockHeader) -> Self {
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
            sync_committee_bits: value
                .sync_committee_bits
                .iter()
                .map(|v| if *v { 1_u8 } else { 0_u8 })
                .collect(),
            sync_committee_signature: value.sync_committee_signature.to_vec(),
        }
    }
}

pub fn ssz_process_sync_committee_period_update(
    prev_update: Vec<u8>,
    update: Vec<u8>,
    validators_root: H256,
) -> Result<(SyncCommittee, BeaconHeader), String> {
    let prev_update: SSZSyncCommitteePeriodUpdate = deserialize(
        &prev_update,
    )
    .map_err(|e| format!("Failed to decode previous update: {}", e))?;
    let update: SSZSyncCommitteePeriodUpdate =
        SSZSyncCommitteePeriodUpdate::deserialize(&update)
            .map_err(|_| "Failed to decode update")?;

    let prev_update = SyncCommitteePeriodUpdate::from(prev_update);
    let update = SyncCommitteePeriodUpdate::from(update);
    process_sync_committee_period_update(prev_update, update, validators_root).map_err(|e| format!("{}", e))
}


pub fn process_sync_committee_period_update(
    prev_update: SyncCommitteePeriodUpdate,
    update: SyncCommitteePeriodUpdate,
    validators_root: H256,
) -> Result<(SyncCommittee, BeaconHeader), String> {
    let sync_committee_bits =
        get_sync_committee_bits(update.sync_aggregate.sync_committee_bits.clone())?;
    //     .map_err(|_| DispatchError::Other("Couldn't process sync committee bits"))?;
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

    Ok((update.next_sync_committee, update.finalized_header))
}

fn get_sync_committee_sum(sync_committee_bits: Vec<u8>) -> u64 {
    sync_committee_bits
        .iter()
        .fold(0, |acc: u64, x| acc + *x as u64)
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

fn get_sync_committee_bits(bits_hex: Vec<u8>) -> Result<Vec<u8>, String> {
    let bitv = Bitvector::<{ SYNC_COMMITTEE_SIZE }>::deserialize(&bits_hex)
        .map_err(|e| format!("DeserializeError: {}", e))?;

    let result = bitv
        .iter()
        .map(|bit| if bit == true { 1 } else { 0 })
        .collect::<Vec<_>>();

    Ok(result)
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

fn hash_tree_root<T: SimpleSerializeTrait>(mut object: T) -> Result<[u8; 32], String> {
    match object.hash_tree_root() {
        Ok(node) => node
            .as_bytes()
            .try_into()
            .map_err(|_| "Invalid hash tree root".into()),
        Err(_e) => Err("MerkleizationError::HashTreeRootError".into()),
    }
}

fn is_valid_merkle_branch(
    leaf: H256,
    branch: Vec<H256>,
    depth: u64,
    index: u64,
    root: Root,
) -> bool {
    if branch.len() != depth as usize {
        // log::error!(target: "ethereum-beacon-client", "Merkle proof branch length doesn't match depth.");

        return false;
    }
    let mut value = leaf;
    if leaf.as_bytes().len() < 32 as usize {
        // log::error!(target: "ethereum-beacon-client", "Merkle proof leaf not 32 bytes.");

        return false;
    }
    for i in 0..depth {
        if branch[i as usize].as_bytes().len() < 32 as usize {
            // log::error!(target: "ethereum-beacon-client", "Merkle proof branch not 32 bytes.");

            return false;
        }
        if (index / (2u32.pow(i as u32) as u64) % 2) == 0 {
            // left node
            let mut data = [0u8; 64];
            data[0..32].copy_from_slice(&(value.0));
            data[32..64].copy_from_slice(&(branch[i as usize].0));
            value = sha2_256(&data).into();
        } else {
            let mut data = [0u8; 64]; // right node
            data[0..32].copy_from_slice(&(branch[i as usize].0));
            data[32..64].copy_from_slice(&(value.0));
            value = sha2_256(&data).into();
        }
    }

    return value == root;
}

fn hash_tree_root_beacon_header(beacon_header: BeaconHeader) -> Result<[u8; 32], String> {
    hash_tree_root(get_ssz_beacon_header(beacon_header)?)
}

fn get_ssz_beacon_header(
    beacon_header: BeaconHeader,
) -> Result<SSZBeaconBlockHeader, String> {
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

fn verify_header(
    block_root: H256,
    proof_branch: Vec<H256>,
    attested_header_state_root: H256,
    depth: u64,
    index: u64,
) -> Result<(), String> {
    if is_valid_merkle_branch(
        block_root,
        proof_branch,
        depth,
        index,
        attested_header_state_root,
    ) {
        return Ok(());
    } else {
        return Err("Header merkle branch is invalid".into());
    }
}
// fn compute_current_sync_period(slot: u64) -> u64 {
//     slot / SLOTS_PER_EPOCH / EPOCHS_PER_SYNC_COMMITTEE_PERIOD
// }

fn verify_signed_header(
    sync_committee_bits: Vec<u8>,
    sync_committee_signature: Vec<u8>,
    sync_committee_pubkeys: Vec<PublicKey>,
    fork_version: ForkVersion,
    header: BeaconHeader,
    validators_root: H256,
) -> Result<(), String> {
    let mut participant_pubkeys: Vec<PublicKey> = Vec::new();
    // Gathers all the pubkeys of the sync committee members that participated in siging the header.
    for (bit, pubkey) in sync_committee_bits
        .iter()
        .zip(sync_committee_pubkeys.iter())
    {
        if *bit == 1 as u8 {
            let pubk = pubkey.clone();
            participant_pubkeys.push(pubk);
        }
    }

    let domain_type = DOMAIN_SYNC_COMMITTEE.to_vec();
    // Domains are used for for seeds, for signatures, and for selecting aggregators.
    let domain = compute_domain(domain_type, Some(fork_version), validators_root)?;
    // Hash tree root of SigningData - object root + domain
    let signing_root = compute_signing_root(header, domain)?;

    // Verify sync committee aggregate signature.
    bls_fast_aggregate_verify(participant_pubkeys, signing_root, sync_committee_signature)?;

    Ok(())
}
fn compute_domain(
    domain_type: Vec<u8>,
    fork_version: Option<ForkVersion>,
    genesis_validators_root: Root,
) -> Result<Domain, String> {
    let unwrapped_fork_version: ForkVersion;
    if fork_version.is_none() {
        unwrapped_fork_version = GENESIS_FORK_VERSION;
    } else {
        unwrapped_fork_version = fork_version.unwrap();
    }

    let fork_data_root = compute_fork_data_root(unwrapped_fork_version, genesis_validators_root)?;

    let mut domain = [0u8; 32];
    domain[0..4].copy_from_slice(&(domain_type));
    domain[4..32].copy_from_slice(&(fork_data_root.0[..28]));

    Ok(domain.into())
}

fn compute_fork_data_root(
    current_version: ForkVersion,
    genesis_validators_root: Root,
) -> Result<Root, String> {
    let hash_root = hash_tree_root_fork_data(ForkData {
        current_version,
        genesis_validators_root: genesis_validators_root.into(),
    })
    .map_err(|_| "Fork data hash tree root failed")?;

    Ok(hash_root.into())
}

fn hash_tree_root_fork_data(fork_data: ForkData) -> Result<[u8; 32], String> {
    hash_tree_root(SSZForkData {
        current_version: fork_data.current_version,
        genesis_validators_root: fork_data.genesis_validators_root,
    })
}

fn compute_signing_root(beacon_header: BeaconHeader, domain: Domain) -> Result<Root, String> {
    let beacon_header_root = hash_tree_root_beacon_header(beacon_header)
        .map_err(|_| "Beacon header hash tree root failed")?;

    let header_hash_tree_root: H256 = beacon_header_root.into();

    let hash_root = hash_tree_root_signing_data(SigningData {
        object_root: header_hash_tree_root,
        domain,
    })
    .map_err(|_| "Signing root hash tree root failed")?;

    Ok(hash_root.into())
}
fn hash_tree_root_signing_data(signing_data: SigningData) -> Result<[u8; 32], String> {
    hash_tree_root(SSZSigningData {
        object_root: signing_data.object_root.into(),
        domain: signing_data.domain.into(),
    })
}

fn bls_fast_aggregate_verify(
    pubkeys: Vec<PublicKey>,
    message: H256,
    signature: Vec<u8>,
) -> Result<(), String> {
    let sig = Signature::from_bytes(&signature[..]);
    if let Err(_e) = sig {
        return Err("InvalidSignature".into());
    }

    let agg_sig = AggregateSignature::from_signature(&sig.unwrap());

    let public_keys_res: Result<Vec<milagro_bls::PublicKey>, _> = pubkeys
        .iter()
        .map(|bytes| milagro_bls::PublicKey::from_bytes_unchecked(&bytes.0))
        .collect();
    if let Err(e) = public_keys_res {
        match e {
            AmclError::InvalidPoint => return Err("InvalidSignaturePoint".into()),
            _ => return Err("InvalidSignature".into()),
        };
    }

    let agg_pub_key_res = AggregatePublicKey::into_aggregate(&public_keys_res.unwrap());
    if let Err(_e) = agg_pub_key_res {
        // log::error!(target: "ethereum-beacon-client", "invalid public keys: {:?}.", e);
        return Err("InvalidAggregatePublicKeys".into());
    }

    if agg_sig.fast_aggregate_verify_pre_aggregated(&message.as_bytes(), &agg_pub_key_res.unwrap())
    {
        Ok(())
    } else {
        Err("SignatureVerificationFailed".into())
    }
}
