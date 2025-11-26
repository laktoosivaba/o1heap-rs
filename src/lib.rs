#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::fmt;
use core::ptr::NonNull;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// Error returned when heap initialization fails.
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

pub struct O1Heap {
    instance: UnsafeCell<*mut O1HeapInstance>,
}

unsafe impl Sync for O1Heap {}

impl O1Heap {
    /// Create a new uninitialized heap.
    ///
    /// You must call [`init`](Self::init) before any allocations.
    pub const fn empty() -> Self {
        Self {
            instance: UnsafeCell::new(core::ptr::null_mut()),
        }
    }

    /// Initialize the heap with the given memory arena.
    ///
    /// # Safety
    ///
    /// - Must be called exactly once before any allocations.
    /// - `start` must be aligned to [`ALIGNMENT`] bytes.
    /// - `start` must point to at least `size` bytes of valid memory.
    /// - The memory must remain valid for the lifetime of the heap.
    pub unsafe fn init(&self, start: *mut u8, size: usize) -> Result<(), InitError> {
        let instance = unsafe { o1heapInit(start.cast(), size) };
        if instance.is_null() {
            return Err(InitError);
        }
        unsafe { *self.instance.get() = instance };
        Ok(())
    }

    #[inline]
    fn get(&self) -> *mut O1HeapInstance {
        unsafe { *self.instance.get() }
    }

    /// Allocate memory of the given size.
    ///
    /// Returns a pointer aligned to [`ALIGNMENT`], or `None` if allocation fails.
    pub fn allocate(&self, size: usize) -> Option<NonNull<u8>> {
        let instance = self.get();
        debug_assert!(!instance.is_null(), "O1Heap not initialized");
        NonNull::new(unsafe { o1heapAllocate(instance, size) }.cast())
    }

    /// Free previously allocated memory.
    ///
    /// # Safety
    ///
    /// - `ptr` must have been returned by [`allocate`](Self::allocate) on this heap.
    /// - `ptr` must not have been freed already.
    pub unsafe fn free(&self, ptr: NonNull<u8>) {
        let instance = self.get();
        debug_assert!(!instance.is_null(), "O1Heap not initialized");
        unsafe { o1heapFree(instance, ptr.as_ptr().cast()) }
    }

    /// Returns the largest contiguous block that can currently be allocated.
    pub fn max_allocation_size(&self) -> usize {
        let instance = self.get();
        debug_assert!(!instance.is_null(), "O1Heap not initialized");
        unsafe { o1heapGetMaxAllocationSize(instance) }
    }

    /// Check if the heap's internal invariants hold.
    pub fn invariants_hold(&self) -> bool {
        let instance = self.get();
        debug_assert!(!instance.is_null(), "O1Heap not initialized");
        unsafe { o1heapDoInvariantsHold(instance) }
    }

    /// Get diagnostic information about the heap.
    pub fn diagnostics(&self) -> O1HeapDiagnostics {
        let instance = self.get();
        debug_assert!(!instance.is_null(), "O1Heap not initialized");
        unsafe { o1heapGetDiagnostics(instance) }
    }
}

/// Returns the minimum arena size required for initialization.
pub fn min_arena_size() -> usize {
    unsafe { o1heapMinArenaSize }
}

unsafe impl GlobalAlloc for O1Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        debug_assert!(
            layout.align() <= ALIGNMENT,
            "o1heap cannot satisfy alignment greater than {}",
            ALIGNMENT
        );
        self.allocate(layout.size())
            .map(|p| p.as_ptr())
            .unwrap_or(core::ptr::null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if let Some(ptr) = NonNull::new(ptr) {
            unsafe { self.free(ptr) }
        }
    }
}
