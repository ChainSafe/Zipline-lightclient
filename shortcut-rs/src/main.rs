use std::io::Write;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;

const PREIMAGE_CACHE_DIR: &str = "../preimage-cache";

const VALIDATORS_ROOT_HEX_STR: &str =
    "4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95";

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let prev_update = load_hash(&args[1]);
    let update = load_hash(&args[2]);

    let validators_root = bacon::H256(hex::decode(VALIDATORS_ROOT_HEX_STR).unwrap().try_into().unwrap());
    let (_sync_committee, _beacon_header) = bacon::ssz_process_sync_committee_period_update(prev_update, update, validators_root)?;

    // println!("{:?}, {:?}", sync_committee, beacon_header);
    let mut stdout = std::io::stdout().lock();

    // write 64 bytes to std-out for testing purposes
    stdout.write_all(&hex::decode(VALIDATORS_ROOT_HEX_STR).unwrap())?;
    stdout.write_all(&hex::decode(VALIDATORS_ROOT_HEX_STR).unwrap())?;

    Ok(())
}

fn load_hash(hash: &str) -> Vec<u8> {
    let mut f = File::open(format!("{}/{}", PREIMAGE_CACHE_DIR, hash)).unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap();
    buffer
}
