use eth_lightclient::SyncCommitteePeriodUpdate;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::io::Write;

const PREIMAGE_CACHE_DIR: &str = "../preimage-cache";

const VALIDATORS_ROOT_HEX_STR: &str =
    "4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95";

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let prev_update = load_hash(&args[1]);
    let update = load_hash(&args[2]);

    let validators_root = eth_lightclient::H256(
        hex::decode(VALIDATORS_ROOT_HEX_STR)
            .unwrap()
            .try_into()
            .unwrap(),
    );
    let _ = eth_lightclient::check_sync_committee_period_update(
        SyncCommitteePeriodUpdate::try_from(prev_update.as_slice()).unwrap(),
        SyncCommitteePeriodUpdate::try_from(update.as_slice()).unwrap(),
        validators_root,
    )?;

    // println!("{:?}, {:?}", sync_committee, beacon_header);
    let stdout = std::io::stdout();
    let mut stdout_lock = stdout.lock();

    // write 64 bytes to std-out for testing purposes
    stdout_lock.write_all(&hex::decode(VALIDATORS_ROOT_HEX_STR).unwrap())?;
    stdout_lock.write_all(&hex::decode(VALIDATORS_ROOT_HEX_STR).unwrap())?;

    //println!("Validation success!!");

    Ok(())
}

fn load_hash(hash: &str) -> Vec<u8> {
    let mut f = File::open(format!("{}/{}", PREIMAGE_CACHE_DIR, hash)).unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap();
    buffer
}
