// By default Wasm module has its own memory. We can specify that instead of using own memory
// host should provide a memory buffer.
// #![feature(wasm_import_memory)]
// #![wasm_import_memory]

mod small;

// This section defines methods that will be provided by host.
extern "C" {
    // Host provides this function
    fn multiply(a: i32, b: i32) -> i32;

    // Host provides this function
    fn state_add(state_ptr: i32, n: i32);
}

// Simple example
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Passing a String from host
// Host sets a string to the webassembly memory directly
#[no_mangle]
pub extern "C" fn count_str(ptr: *const u8, len: usize) -> usize {
    use std::slice;
    use std::str;

    let s = unsafe {
        // First, we build a &[u8]...
        let slice = slice::from_raw_parts(ptr, len);
        // ... and then convert that slice into a string slice
        // str::from_utf8_unchecked(slice)
        str::from_utf8(slice)
    };

    s.unwrap().len()
}

// Returning a String
// https://www.hellorust.com/demos/sha1/index.html
use std::os::raw::c_char;
#[no_mangle]
pub extern "C" fn return_str() -> *mut c_char {
    use std::ffi::CString;

    let hello: String = "hello from webassembly".into();
    let s = CString::new(hello).unwrap();
    s.into_raw()
}

// Calling a host function
#[no_mangle]
pub extern "C" fn calc_host(a: i32) -> i32 {
    let b = a + a;
    unsafe { multiply(a, b) }
}

// Passing a complex object from host and calling it's method
#[no_mangle]
pub extern "C" fn host_state_add(state_ptr: i32) {
    unsafe {
        state_add(state_ptr, 5);
    }
}
