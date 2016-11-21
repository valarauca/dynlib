
#![crate_type="dylib"]

#[no_mangle]
pub extern "C" fn addition(x: u64, y: u64) -> u64 {
    x + y
}
