// Ethereum Mainnet beacon chain constants
pub mod mainnet {
    use crate::types::ForkVersion;

    pub const SYNC_COMMITTEE_SIZE: usize = 512;
    pub const PUBKEY_SIZE: usize = 48;
    pub const SIGNATURE_SIZE: usize = 96;
    pub const NEXT_SYNC_COMMITTEE_DEPTH: u64 = 5;
    pub const NEXT_SYNC_COMMITTEE_INDEX: u64 = 23;
    pub const FINALIZED_ROOT_DEPTH: u64 = 6;
    pub const FINALIZED_ROOT_INDEX: u64 = 41;
    pub const DOMAIN_SYNC_COMMITTEE: [u8; 4] = [7, 0, 0, 0];
    pub const GENESIS_FORK_VERSION: ForkVersion = [30, 30, 30, 30];

    pub const VALIDATORS_ROOT: [u8; 32] = [
        75, 54, 61, 185, 78, 40, 97, 32, 215, 110, 185, 5, 52, 15, 221, 78, 84, 191, 233, 240, 107,
        243, 63, 246, 207, 90, 210, 127, 81, 27, 254, 149,
    ];
}
