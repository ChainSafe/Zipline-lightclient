use eth_lightclient;

const VALIDATORS_ROOT: [u8; 32] = [
    75, 54, 61, 185, 78, 40, 97, 32, 215, 110, 185, 5, 52, 15, 221, 78, 84, 191, 233, 240, 107,
    243, 63, 246, 207, 90, 210, 127, 81, 27, 254, 149,
];

const A: &[u8] = include_bytes!(
    "sync-updates/0xe4c2cee3a9455c2b7c0449152a8c7e1a7b811353e4ea2c1dbe1cbe0c790b45f7"
);
const B: &[u8] = include_bytes!(
    "sync-updates/0x78ae69239826edd5ac0abfe3a69e916e7479ad44e834e35a08e4df7601732a85"
);

#[test]
fn can_check_valid_transition() -> Result<(), String> {
    eth_lightclient::check_sync_committee_period_update(
        eth_lightclient::SyncCommitteePeriodUpdate::try_from(A).unwrap(),
        eth_lightclient::SyncCommitteePeriodUpdate::try_from(B).unwrap(),
        eth_lightclient::H256(VALIDATORS_ROOT),
    )
}

#[test]
fn can_check_invalid_transition() {

    let mut b_fail = B.to_vec();
    b_fail[100] = 0x00; // zero out a random byte

    assert_eq!(
        eth_lightclient::check_sync_committee_period_update(
            eth_lightclient::SyncCommitteePeriodUpdate::try_from(A).unwrap(),
            eth_lightclient::SyncCommitteePeriodUpdate::try_from(b_fail.as_slice()).unwrap(),
            eth_lightclient::H256(VALIDATORS_ROOT),
        ),
        Err("SignatureVerificationFailed".to_string())
    )
}
