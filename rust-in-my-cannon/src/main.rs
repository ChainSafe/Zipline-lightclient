//! This program is compiled into MIPS and is used for the onchain verification.
//! 
//! It just wraps the `arbitrary-state-machine` crate providing the necessary interface with the 
//! host (onchain verfier or offchain prover).

#![feature(alloc_error_handler)] // no_std and allocator support is not stable.
#![feature(stdsimd)] // for `mips::break_`. If desired, this could be replaced with asm.
#![feature(restricted_std)]
#![no_main]

extern crate alloc;
extern crate rlibc; // memcpy, and friends

mod heap;
mod iommu;

pub type H256 = [u8; 32];

/// Main entrypoint.
#[no_mangle]
pub extern "C" fn _start() {
    unsafe { heap::init() };

    // let mut x = 0;
    // loop {
    //     x += 1;
    // }
    
    let mut v = vec![1_u8,2_u8,3_u8];
    v[0] += 1;

    iommu::output([v[0]; 32]);

}

// #[panic_handler]
// fn panic(info: &core::panic::PanicInfo) -> ! {
//     // Uncomment code below if you're in trouble
//     /* 
//     let msg = alloc::format!("Panic: {}", info);
//     iommu::print(&msg);
//     */ 

//     unsafe {
//         core::arch::mips::break_();
//     }
// }

// #[alloc_error_handler]
// fn alloc_error_handler(_layout: alloc::alloc::Layout) -> ! {
//     // NOTE: avoid `panic!` here, technically, it might not be allowed to panic in an OOM situation.
//     //       with panic=abort it should work, but it's no biggie use `break` here anyway.
//     unsafe {
//         core::arch::mips::break_();
//     }
// }
