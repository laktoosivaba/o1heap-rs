//! # o1heap
//!
//! Rust bindings for [o1heap](https://github.com/pavel-kirienko/o1heap) -
//! a constant-time deterministic memory allocator for hard real-time systems.
//!
//! ## Features
//!
//! - **Constant-time** allocation and deallocation (O(1))
//! - **Deterministic** behavior suitable for real-time systems
//! - **No-std** compatible
//! - **Bounded fragmentation**
//!
//! ## Example
//!
//! ```ignore
//! use o1heap::O1Heap;
//! use core::mem::MaybeUninit;
//!
//! // Create aligned memory arena
//! #[repr(C, align(16))]
//! struct Arena([MaybeUninit<u8>; 4096]);
//!
//! static mut ARENA: Arena = Arena([MaybeUninit::uninit(); 4096]);
//!
//! let heap = unsafe {
//!     O1Heap::new(ARENA.0.as_mut_ptr() as *mut _, 4096)
//! }.expect("Failed to initialize heap");
//!
//! let ptr = heap.allocate(64);
//! if !ptr.is_null() {
//!     // Use allocated memory...
//!     unsafe { heap.free(ptr) };
//! }
//! ```

#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]

use core::ffi::c_void;
use core::fmt;

// Include generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// Error returned when heap initialization fails.
///
/// This can happen if:
/// - The base pointer is null
/// - The base pointer is not aligned to [`ALIGNMENT`]
/// - The arena size is less than [`min_arena_size()`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitError;

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "o1heap initialization failed: arena too small or misaligned"
        )
    }
}

/// The guaranteed alignment of allocated memory (platform-dependent).
/// On 32-bit systems: 16 bytes. On 64-bit systems: 32 bytes.
pub const ALIGNMENT: usize = core::mem::size_of::<*const ()>() * 4;

/// Safe wrapper around o1heap instance.
pub struct O1Heap {
    instance: *mut O1HeapInstance,
}

// O1Heap is not thread-safe; users must provide external synchronization
// if used across threads.
unsafe impl Send for O1Heap {}

impl O1Heap {
    /// Create a new O1Heap instance from a memory arena.
    ///
    /// # Safety
    /// - `base` must be aligned to [`ALIGNMENT`] bytes
    /// - `base` must point to at least `size` bytes of valid memory
    /// - The memory must remain valid for the lifetime of this O1Heap
    ///
    /// # Errors
    /// Returns `None` if:
    /// - `base` is null
    /// - `base` is not properly aligned
    /// - `size` is too small (less than [`min_arena_size()`])
    pub unsafe fn new(base: *mut c_void, size: usize) -> Option<Self> {
        let instance = unsafe { o1heapInit(base, size) };
        if instance.is_null() {
            None
        } else {
            Some(Self { instance })
        }
    }

    /// Allocate memory of the given size.
    ///
    /// Returns a pointer aligned to [`ALIGNMENT`], or null if allocation fails.
    /// The allocation is performed in constant time.
    pub fn allocate(&self, size: usize) -> *mut c_void {
        unsafe { o1heapAllocate(self.instance, size) }
    }

    /// Free previously allocated memory.
    ///
    /// # Safety
    /// - `ptr` must have been returned by a previous call to [`allocate`](Self::allocate)
    ///   on this same heap instance, or be null.
    /// - `ptr` must not have been freed already.
    pub unsafe fn free(&self, ptr: *mut c_void) {
        unsafe { o1heapFree(self.instance, ptr) }
    }

    /// Get the maximum allocation size for this heap.
    pub fn max_allocation_size(&self) -> usize {
        unsafe { o1heapGetMaxAllocationSize(self.instance) }
    }

    /// Check if the heap's internal invariants hold.
    ///
    /// Useful for runtime self-diagnostics.
    pub fn invariants_hold(&self) -> bool {
        unsafe { o1heapDoInvariantsHold(self.instance) }
    }

    /// Get diagnostic information about the heap.
    pub fn diagnostics(&self) -> O1HeapDiagnostics {
        unsafe { o1heapGetDiagnostics(self.instance) }
    }
}

/// Returns the minimum arena size required for initialization.
pub fn min_arena_size() -> usize {
    unsafe { o1heapMinArenaSize }
}

// Optional: GlobalAlloc implementation behind a feature flag
#[cfg(feature = "global_alloc")]
mod global_alloc;

#[cfg(feature = "global_alloc")]
pub use global_alloc::O1HeapGlobalAlloc;
