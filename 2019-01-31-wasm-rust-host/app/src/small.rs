// #![no_std]

// Replacing the allocator and using the `alloc` crate are still unstable.
// #![feature(alloc, core_intrinsics, lang_items, alloc_error_handler)]

// #[panic_handler]
// #[no_mangle]
// pub fn panic(_info: &::core::panic::PanicInfo) -> ! {
//     unsafe {
//         ::core::intrinsics::abort();
//     }
// }

// // Need to provide an allocation error handler which just aborts
// // the execution with trap.
// #[alloc_error_handler]
// #[no_mangle]
// pub extern "C" fn oom(_: ::core::alloc::Layout) -> ! {
//     unsafe {
//         ::core::intrinsics::abort();
//     }
// }

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        use wee_alloc;
        // Use `wee_alloc` as the global allocator.
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}
