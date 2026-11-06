// Test-thread requested-byte heap peaks. Child threads have independent peaks.
// libtest starts a fresh named thread for each test, including serial test runs.
#![deny(unsafe_op_in_unsafe_fn)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};

type Report = fn(&str, usize);

/// A System-backed counting allocator for native test executables.
/// The callback runs during each qualified named thread's teardown, with heap
/// tracking suppressed so reporting does not affect the measured peak.
pub struct HeapAllocator {
    report: Report,
}
impl HeapAllocator {
    pub const fn new(report: Report) -> Self {
        Self { report }
    }
}

#[cfg(test)]
#[global_allocator]
static ALLOCATOR: HeapAllocator = HeapAllocator::new(|_, _| {});

struct Tracker {
    live: AtomicUsize,
    peak: AtomicUsize,
}
impl Tracker {
    const fn new() -> Self {
        Self { live: AtomicUsize::new(0), peak: AtomicUsize::new(0) }
    }
    fn add(&self, bytes: usize) {
        let live = self.live.fetch_add(bytes, Ordering::Relaxed) + bytes;
        self.peak.fetch_max(live, Ordering::Relaxed);
    }
    fn remove(&self, bytes: usize) {
        self.live.fetch_sub(bytes, Ordering::Relaxed);
    }
}

struct ThreadHeap {
    tracker: *mut Tracker,
    thread: std::thread::Thread,
    report: Cell<Option<Report>>,
}
std::thread_local! {
    // This non-dropping guard remains available during TLS destruction.
    static SUPPRESSED: Cell<bool> = const { Cell::new(false) };
    static HEAP: ThreadHeap = ThreadHeap::new();
}
impl ThreadHeap {
    fn new() -> Self {
        // SAFETY: instrumentation storage uses System directly, has Tracker's
        // layout, and is initialized before publication. It intentionally lives
        // until process exit: allocations can be freed after their thread exits.
        let tracker = unsafe { System.alloc(Layout::new::<Tracker>()).cast::<Tracker>() };
        if tracker.is_null() {
            std::process::abort();
        }
        unsafe { tracker.write(Tracker::new()) };
        Self { tracker, thread: std::thread::current(), report: Cell::new(None) }
    }
}
impl Drop for ThreadHeap {
    fn drop(&mut self) {
        // Capture the name while the retained handle is still valid. Reporting
        // allocations must not initialize another tracker or affect this peak.
        without_tracking(|| {
            if let Some(test) = self.thread.name().filter(|name| name.contains("::")) {
                // SAFETY: tracker storage remains valid until process exit.
                let peak = unsafe { &*self.tracker }.peak.load(Ordering::Relaxed);
                if let Some(report) = self.report.get() {
                    report(test, peak);
                }
            }
        });
    }
}

/// Suspends this thread's heap accounting until dropped; guards nest. Blocks
/// allocated meanwhile are never charged to the thread, even after they outlive
/// the guard, so diagnostic capture does not change a test's measured peak.
pub struct Untracked(bool, std::marker::PhantomData<*const ()>);
impl Drop for Untracked {
    fn drop(&mut self) {
        let _ = SUPPRESSED.try_with(|flag| flag.set(self.0));
    }
}
pub fn untracked() -> Untracked {
    let previous = SUPPRESSED.try_with(|flag| flag.replace(true)).unwrap_or(true);
    Untracked(previous, std::marker::PhantomData)
}

fn without_tracking<T>(f: impl FnOnce() -> T) -> T {
    let _untracked = untracked();
    f()
}

fn owner(report: Report) -> *mut Tracker {
    // Only first-use initialization obtains a thread handle. The normal path
    // performs TLS lookups and atomic accounting, with no locks or environment IO.
    SUPPRESSED
        .try_with(|flag| {
            if flag.replace(true) {
                return ptr::null_mut();
            }
            let tracker = HEAP
                .try_with(|heap| {
                    if heap.report.get().is_none() {
                        heap.report.set(Some(report));
                    }
                    heap.tracker
                })
                .unwrap_or(ptr::null_mut());
            flag.set(false);
            tracker
        })
        .unwrap_or(ptr::null_mut())
}

#[derive(Clone, Copy)]
struct Header {
    owner: *mut Tracker,
}
fn allocation_layout(layout: Layout) -> Option<(Layout, usize)> {
    let (combined, offset) = Layout::new::<Header>().extend(layout).ok()?;
    Some((combined.pad_to_align(), offset))
}

// SAFETY: each returned pointer satisfies the requested alignment and has a
// private, initialized header. All System calls receive the matching expanded
// layout; payload sizes alone count toward the test's peak. Header owners stay
// alive across threads and TLS destruction. No allocator path unwinds.
unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: forwarded allocator contract; helper initializes the header.
        unsafe { allocate(layout, false, owner(self.report)) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: System zeroes the expanded block, including the whole payload.
        unsafe { allocate(layout, true, owner(self.report)) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        let Some((expanded, offset)) = allocation_layout(layout) else { std::process::abort() };
        // SAFETY: pointer came from allocate with this layout; its header is
        // offset bytes before it and its owner remains alive even on another thread.
        unsafe {
            let base = pointer.sub(offset);
            let header = base.cast::<Header>().read();
            if let Some(tracker) = header.owner.as_ref() {
                tracker.remove(layout.size());
            }
            System.dealloc(base, expanded);
        }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let Ok(new_layout) = Layout::from_size_align(new_size, layout.align()) else {
            return ptr::null_mut();
        };
        let Some((old_expanded, offset)) = allocation_layout(layout) else {
            return ptr::null_mut();
        };
        let Some((new_expanded, new_offset)) = allocation_layout(new_layout) else {
            return ptr::null_mut();
        };
        // SAFETY: unchanged payload alignment implies unchanged header offset
        // and expanded alignment. A failed realloc leaves the old block/count intact.
        unsafe {
            let base = pointer.sub(offset);
            let header = base.cast::<Header>().read();
            let replacement = System.realloc(base, old_expanded, new_expanded.size());
            if replacement.is_null() {
                return ptr::null_mut();
            }
            if let Some(tracker) = header.owner.as_ref() {
                if new_size >= layout.size() {
                    tracker.add(new_size - layout.size());
                } else {
                    tracker.remove(layout.size() - new_size);
                }
            }
            replacement.add(new_offset)
        }
    }
}
unsafe fn allocate(layout: Layout, zeroed: bool, tracker: *mut Tracker) -> *mut u8 {
    let Some((expanded, offset)) = allocation_layout(layout) else { return ptr::null_mut() };
    // SAFETY: expanded is a valid Layout; write the header only after successful
    // allocation, then return its aligned payload. System never uses this wrapper.
    unsafe {
        let base = if zeroed { System.alloc_zeroed(expanded) } else { System.alloc(expanded) };
        if base.is_null() {
            return ptr::null_mut();
        }
        base.cast::<Header>().write(Header { owner: tracker });
        if let Some(tracker) = tracker.as_ref() {
            tracker.add(layout.size());
        }
        base.add(offset)
    }
}

#[test]
fn allocation_alignment_zeroing_and_reallocation_preserve_live_peak() {
    let tracker = Tracker::new();
    let owner = &tracker as *const Tracker as *mut Tracker;
    let layout = Layout::from_size_align(37, 256).unwrap();
    // SAFETY: every pointer below comes from this allocator with the supplied
    // layout, and the tracker outlives all of its blocks.
    unsafe {
        let pointer = allocate(layout, true, owner);
        assert!(!pointer.is_null());
        assert_eq!(pointer.addr() % 256, 0);
        assert!(std::slice::from_raw_parts(pointer, 37).iter().all(|byte| *byte == 0));
        pointer.write(42);
        assert_eq!(tracker.live.load(Ordering::Relaxed), 37);
        let pointer = ALLOCATOR.realloc(pointer, layout, 83);
        assert!(!pointer.is_null());
        assert_eq!(pointer.addr() % 256, 0);
        assert_eq!(pointer.read(), 42);
        let grown = Layout::from_size_align(83, 256).unwrap();
        assert!(ALLOCATOR.realloc(pointer, grown, usize::MAX).is_null());
        // This valid layout exceeds native address space: System must fail
        // without consuming memory or destroying the original allocation.
        assert!(ALLOCATOR.realloc(pointer, grown, isize::MAX as usize / 2).is_null());
        assert_eq!(tracker.live.load(Ordering::Relaxed), 83);
        assert_eq!(pointer.read(), 42);
        let pointer = ALLOCATOR.realloc(pointer, grown, 11);
        assert!(!pointer.is_null());
        assert_eq!(tracker.live.load(Ordering::Relaxed), 11);
        assert_eq!(tracker.peak.load(Ordering::Relaxed), 83);
        ALLOCATOR.dealloc(pointer, Layout::from_size_align(11, 256).unwrap());
    }
    assert_eq!(tracker.live.load(Ordering::Relaxed), 0);
}

#[test]
fn deallocation_on_another_thread_debits_the_original_owner() {
    let tracker = std::boxed::Box::leak(std::boxed::Box::new(Tracker::new()));
    // SAFETY: an initialized allocation of exactly this array's layout is handed
    // to Box, which deallocates it through the same global allocator on the child.
    let value = unsafe {
        let pointer = allocate(Layout::new::<[u8; 19]>(), true, tracker);
        assert!(!pointer.is_null());
        std::boxed::Box::from_raw(pointer.cast::<[u8; 19]>())
    };
    std::thread::spawn(move || drop(value)).join().unwrap();
    assert_eq!(tracker.live.load(Ordering::Relaxed), 0);
    assert_eq!(tracker.peak.load(Ordering::Relaxed), 19);
}

#[test]
fn owner_storage_survives_its_thread_and_suppression_does_not_recurse() {
    let value = std::thread::spawn(|| std::boxed::Box::new([7_u8; 41])).join().unwrap();
    assert_eq!(value[0], 7);
    drop(value);
    without_tracking(|| {
        assert!(owner(ALLOCATOR.report).is_null());
        without_tracking(|| assert!(owner(ALLOCATOR.report).is_null()));
        assert!(owner(ALLOCATOR.report).is_null());
    });
    assert!(!owner(ALLOCATOR.report).is_null());
}

#[test]
fn peak_counts_overlapping_live_payloads_instead_of_total_allocated_bytes() {
    let tracker = Tracker::new();
    let owner = &tracker as *const Tracker as *mut Tracker;
    let layout = Layout::from_size_align(23, 1).unwrap();
    // SAFETY: the tracker outlives the correctly paired allocations/deallocations.
    unsafe {
        let first = allocate(layout, false, owner);
        let second = allocate(layout, false, owner);
        assert!(!first.is_null() && !second.is_null());
        assert_eq!(tracker.live.load(Ordering::Relaxed), 46);
        ALLOCATOR.dealloc(first, layout);
        ALLOCATOR.dealloc(second, layout);
        let third = allocate(layout, false, owner);
        assert!(!third.is_null());
        assert_eq!(tracker.live.load(Ordering::Relaxed), 23);
        assert_eq!(tracker.peak.load(Ordering::Relaxed), 46);
        ALLOCATOR.dealloc(third, layout);
    }
}
