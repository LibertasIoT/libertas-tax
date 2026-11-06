// Writer capability marker used by the external gate: TAX_TEST_PEAK_HEAP_STATS.
// Unsafe allocator mechanics live in the dev-only helper crate; the App retains
// its crate-wide prohibition on unsafe code.
#[global_allocator]
static ALLOCATOR: tax_test_heap::HeapAllocator =
    tax_test_heap::HeapAllocator::new(super::cross::oracle::monetary_capture::report_heap);
