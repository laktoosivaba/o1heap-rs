//! Example demonstrating O1Heap as a global allocator in a no_std embedded context.
//!
//! This example is meant as a reference and will not compile without
//! a target-specific runtime (like cortex-m-rt).

#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use core::mem::{size_of, MaybeUninit};
use core::panic::PanicInfo;
use core::ptr::addr_of_mut;
use cortex_m_rt::entry;
use o1heap::O1Heap;

#[repr(C, align(16))]
struct Arena([MaybeUninit<u8>; 8192]);
static mut ARENA: Arena = Arena([MaybeUninit::uninit(); 8192]);

#[global_allocator]
static HEAP: O1Heap = O1Heap::empty();

#[entry]
fn main() -> ! {
    unsafe {
        HEAP.init(addr_of_mut!(ARENA.0).cast(), size_of::<Arena>())
            .expect("heap init failed");
    }

    let mut xs = Vec::new();
    xs.push(1);
    xs.push(2);
    xs.push(3);

    #[allow(clippy::empty_loop)]
    loop { /* .. */ }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}