//! This program is compiled into MIPS and is used for the onchain verification.
//!
//! It just wraps the `arbitrary-state-machine` crate providing the necessary interface with the
//! host (onchain verfier or offchain prover).

#![feature(alloc_error_handler)] // no_std and allocator support is not stable.
#![feature(stdsimd)] // for `mips::break_`. If desired, this could be replaced with asm.
#![no_std]
#![no_main]

extern crate alloc;
extern crate rlibc; // memcpy, and friends

mod heap;
mod iommu;

pub type H256 = [u8; 32];

const VALIDATORS_ROOT: H256 = [
    75, 54, 61, 185, 78, 40, 97, 32, 215, 110, 185, 5, 52, 15, 221, 78, 84, 191, 233, 240, 107,
    243, 63, 246, 207, 90, 210, 127, 81, 27, 254, 149,
];

/// Main entrypoint.
#[no_mangle]
pub extern "C" fn _start() {
    unsafe { heap::init() };

    // grab the inputs by hash
    let input_hash_a = iommu::input_hash_A();
    let input_hash_b = iommu::input_hash_B();

    let prev_update_bytes = iommu::preimage(input_hash_a).unwrap();
    let current_update_bytes = iommu::preimage(input_hash_b).unwrap();

    match eth_lightclient::check_sync_committee_period_update(
        eth_lightclient::SyncCommitteePeriodUpdate::try_from(prev_update_bytes).unwrap(),
        eth_lightclient::SyncCommitteePeriodUpdate::try_from(current_update_bytes).unwrap(),
        eth_lightclient::H256(VALIDATORS_ROOT),
    ) {
        Ok(_) => {
            iommu::output([0xff_u8; 32]);
        }
        Err(_) => {
            iommu::output([0x00_u8; 32]);
        }
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Uncomment code below if you're in trouble
    /*
    let msg = alloc::format!("Panic: {}", info);
    iommu::print(&msg);
    */

    unsafe {
        core::arch::mips::break_();
    }
}

#[alloc_error_handler]
fn alloc_error_handler(_layout: alloc::alloc::Layout) -> ! {
    // NOTE: avoid `panic!` here, technically, it might not be allowed to panic in an OOM situation.
    //       with panic=abort it should work, but it's no biggie use `break` here anyway.
    unsafe {
        core::arch::mips::break_();
    }
}
