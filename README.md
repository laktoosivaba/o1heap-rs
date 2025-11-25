# o1heap-rs

Rust bindings for [o1heap](https://github.com/pavel-kirienko/o1heap) - a constant-time deterministic memory allocator for hard real-time systems.

## Features

- **Constant-time O(1)** allocation and deallocation
- **Deterministic** behavior suitable for real-time systems
- **No-std** compatible - designed for embedded systems
- **Bounded fragmentation** - predictable memory usage
- **Safe Rust wrapper** with optional `GlobalAlloc` implementation

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
o1heap = "0.0.1"
```

For use as a global allocator:

```toml
[dependencies]
o1heap = { version = "0.0.1", features = ["global_alloc"] }
```

## Usage

### Basic Usage

```rust
use o1heap::O1Heap;
use core::mem::MaybeUninit;

// Create an aligned memory arena
#[repr(C, align(16))]
struct Arena([MaybeUninit<u8>; 4096]);

static mut ARENA: Arena = Arena([MaybeUninit::uninit(); 4096]);

fn main() {
    let heap = unsafe {
        O1Heap::new(ARENA.0.as_mut_ptr() as *mut _, 4096)
    }.expect("Failed to initialize heap");

    // Allocate memory
    let ptr = heap.allocate(64);
    if !ptr.is_null() {
        // Use allocated memory...
        
        // Free when done
        unsafe { heap.free(ptr) };
    }
}
```

### As Global Allocator

```rust
#![no_std]

extern crate alloc;

use o1heap::O1HeapGlobalAlloc;
use core::mem::MaybeUninit;

#[repr(C, align(16))]
struct Arena([MaybeUninit<u8>; 8192]);
static mut ARENA: Arena = Arena([MaybeUninit::uninit(); 8192]);

#[global_allocator]
static HEAP: O1HeapGlobalAlloc = O1HeapGlobalAlloc::new();

fn init_heap() {
    unsafe { 
        HEAP.init(ARENA.0.as_mut_ptr() as *mut _, 8192) 
    }.expect("heap init failed");
}

fn main() {
    init_heap();
    
    // Now you can use alloc types like Box, Vec, etc.
    let boxed = alloc::boxed::Box::new(42);
}
```

## Alignment

o1heap guarantees that all allocated memory is aligned to `O1HEAP_ALIGNMENT`:
- **32-bit systems**: 16 bytes
- **64-bit systems**: 32 bytes

This alignment is available as `o1heap::ALIGNMENT`.

## Minimum Arena Size

The minimum arena size can be queried at runtime with `o1heap::min_arena_size()`. This is typically around 192 bytes on 32-bit systems.

## Diagnostics

```rust
let diag = heap.diagnostics();
println!("Capacity: {}", diag.capacity);
println!("Allocated: {}", diag.allocated);
println!("Peak allocated: {}", diag.peak_allocated);
println!("OOM count: {}", diag.oom_count);

// Runtime invariant check
assert!(heap.invariants_hold());
```

## Thread Safety

o1heap is **not thread-safe**. If you need to use it from multiple threads or interrupt contexts, you must provide external synchronization (e.g., critical sections, mutexes).

## License

This crate is licensed under the MIT license. The underlying [o1heap](https://github.com/pavel-kirienko/o1heap) C library is Copyright (c) Pavel Kirienko and also MIT licensed.

## Credits

- [o1heap](https://github.com/pavel-kirienko/o1heap) by Pavel Kirienko