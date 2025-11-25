//! GlobalAlloc implementation for use as #[global_allocator]

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ffi::c_void;

use crate::{o1heapAllocate, o1heapFree, o1heapInit, InitError, O1HeapInstance, ALIGNMENT};

/// A global allocator backed by o1heap.
///
/// # Example
///
/// ```ignore
/// use o1heap::O1HeapGlobalAlloc;
/// use core::mem::MaybeUninit;
///
/// #[repr(C, align(16))]
/// struct Arena([MaybeUninit<u8>; 8192]);
/// static mut ARENA: Arena = Arena([MaybeUninit::uninit(); 8192]);
///
/// #[global_allocator]
/// static HEAP: O1HeapGlobalAlloc = O1HeapGlobalAlloc::new();
///
/// // Call before any allocations:
/// unsafe { HEAP.init(ARENA.0.as_mut_ptr() as *mut _, 8192) }.unwrap();
/// ```
pub struct O1HeapGlobalAlloc {
    instance: UnsafeCell<*mut O1HeapInstance>,
}

// SAFETY: O1HeapGlobalAlloc is designed for single-core embedded systems.
// The heap pointer is set once during initialization before any allocations,
// and then only read during alloc/dealloc operations.
// Users must provide external synchronization if used in multi-threaded contexts.
unsafe impl Sync for O1HeapGlobalAlloc {}

impl Default for O1HeapGlobalAlloc {
    fn default() -> Self {
        Self::new()
    }
}

impl O1HeapGlobalAlloc {
    /// Create a new uninitialized global allocator.
    ///
    /// You must call [`init`](Self::init) before any allocations are made.
    pub const fn new() -> Self {
        Self {
            instance: UnsafeCell::new(core::ptr::null_mut()),
        }
    }

    /// Initialize the allocator with the given memory arena.
    ///
    /// # Safety
    /// - Must be called exactly once before any allocations.
    /// - `base` must be aligned to [`ALIGNMENT`] bytes.
    /// - `base` must point to at least `size` bytes of valid memory.
    /// - The memory must remain valid for the lifetime of the program.
    ///
    /// # Errors
    /// Returns [`InitError`] if o1heapInit fails (arena too small or misaligned).
    pub unsafe fn init(&self, base: *mut c_void, size: usize) -> Result<(), InitError> {
        let instance = unsafe { o1heapInit(base, size) };
        if instance.is_null() {
            return Err(InitError);
        }
        unsafe { *self.instance.get() = instance };
        Ok(())
    }

    /// Get the underlying heap instance pointer.
    #[inline]
    fn get(&self) -> *mut O1HeapInstance {
        unsafe { *self.instance.get() }
    }
}

unsafe impl GlobalAlloc for O1HeapGlobalAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // o1heap always returns ALIGNMENT-aligned memory.
        // For larger alignment requirements, users need a different approach.
        debug_assert!(
            layout.align() <= ALIGNMENT,
            "o1heap cannot satisfy alignment greater than {}",
            ALIGNMENT
        );
        unsafe { o1heapAllocate(self.get(), layout.size()) as *mut u8 }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe { o1heapFree(self.get(), ptr as *mut c_void) }
    }
}
