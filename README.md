# o1heap-rs

Rust bindings for [o1heap](https://github.com/pavel-kirienko/o1heap) - a constant-time deterministic memory allocator for hard real-time systems.

## Features

- **Constant-time O(1)** allocation and deallocation
- **Deterministic** behavior suitable for real-time systems
- **No-std** compatible - designed for embedded systems
- **Bounded fragmentation** - predictable memory usage

## Installation

```toml
[dependencies]
o1heap = "0.0.2"
```

## Usage

### As Global Allocator (STM32)

```rust
#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use core::mem::{size_of, MaybeUninit};
use core::ptr::addr_of_mut;
use cortex_m_rt::entry;
use o1heap::O1Heap;
use panic_halt as _;

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

    loop { }
}
```

## Alignment

o1heap guarantees that all allocated memory is aligned to:
- **32-bit systems**: 16 bytes
- **64-bit systems**: 32 bytes

This alignment is available as `o1heap::ALIGNMENT`.

## Diagnostics

```rust
let diag = heap.diagnostics();
println!("Capacity: {}", diag.capacity);
println!("Allocated: {}", diag.allocated);
println!("Peak allocated: {}", diag.peak_allocated);
println!("OOM count: {}", diag.oom_count);

assert!(heap.invariants_hold());
```

## Thread Safety

o1heap is **not thread-safe**. If you need to use it from multiple threads or interrupt contexts, you must provide external synchronization (e.g., critical sections, mutexes).

## License
This crate is licensed under the MIT license. The underlying [o1heap](https://github.com/pavel-kirienko/o1heap) C library is Copyright (c) Pavel Kirienko and also MIT licensed.

## Credits
- [o1heap](https://github.com/pavel-kirienko/o1heap) by Pavel Kirienko
- [embedded-alloc](https://github.com/rust-embedded/embedded-alloc) - crate API inspiration