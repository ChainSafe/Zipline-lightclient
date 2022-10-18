pub use milagro_bls::{AggregatePublicKey, AggregateSignature, AmclError, Signature};
use sha2::Digest;
use sha2::Sha256;
use crate::types::*;
use crate::types::{get_ssz_beacon_header};

use alloc::vec::Vec;
use alloc::string::String;

pub use ssz_rs::{
    prelude::Vector, Bitvector, Deserialize, SimpleSerialize as SimpleSerializeTrait, Sized,
};

use crate::constants::*;

fn sha2_256(data: &[u8]) -> H256 {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut h = H256::default();
    h.0.copy_from_slice(&result);
    h
}

pub(super) fn get_sync_committee_bits(bitv: Bitvector::<{ SYNC_COMMITTEE_SIZE }>) -> Result<Vec<u8>, String> {
    let result = bitv
        .iter()
        .map(|bit| if bit == true { 1 } else { 0 })
        .collect::<Vec<_>>();
    Ok(result)
}

pub(super)fn get_sync_committee_sum(sync_committee_bits: Vec<u8>) -> u64 {
    sync_committee_bits
        .iter()
        .fold(0, |acc: u64, x| acc + *x as u64)
}

pub(super) fn hash_tree_root_beacon_header(beacon_header: BeaconBlockHeader) -> Result<[u8; 32], String> {
    hash_tree_root(get_ssz_beacon_header(beacon_header)?)
}


pub(super) fn hash_tree_root<T: SimpleSerializeTrait>(mut object: T) -> Result<[u8; 32], String> {
    match object.hash_tree_root() {
        Ok(node) => node
            .as_bytes()
            .try_into()
            .map_err(|_| "Invalid hash tree root".into()),
        Err(_e) => Err("MerkleizationError::HashTreeRootError".into()),
    }
}

pub(super) fn verify_header(
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

pub(super)fn verify_signed_header(
    sync_committee_bits: Vec<u8>,
    sync_committee_signature: Vec<u8>,
    sync_committee_pubkeys: Vec<PublicKey>,
    fork_version: ForkVersion,
    header: BeaconBlockHeader,
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

pub(super)fn is_valid_merkle_branch(
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

pub(super) fn compute_domain(
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

pub(super)fn compute_fork_data_root(
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

pub(super)fn hash_tree_root_fork_data(fork_data: ForkData) -> Result<[u8; 32], String> {
    hash_tree_root(SSZForkData {
        current_version: fork_data.current_version,
        genesis_validators_root: fork_data.genesis_validators_root,
    })
}

pub(super)fn bls_fast_aggregate_verify(
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

fn compute_signing_root(beacon_header: BeaconBlockHeader, domain: Domain) -> Result<Root, String> {
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
