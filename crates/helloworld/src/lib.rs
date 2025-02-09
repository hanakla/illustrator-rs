// use illustrator_rs;

use std::ffi::{c_char, CStr};
use String;

struct Plugin {}

impl Plugin {
    pub fn new() -> Self {
        Plugin {}
    }
}

extern "C" fn PluginMain(
    caller: *mut c_char,
    selector: *mut c_char,
    message: *mut c_char,
) -> illustrator_rs::ASErr {
    // if !caller.is_null() {
    //     let caller = String::from(caller);
    //     println!("Caller: {}", caller);
    // }

    return 0;
}

// #[no_mangle]
// impl From<*mut c_char> for String {
//     fn from(ptr: *mut c_char) -> Self {
//         unsafe {
//             let c_str = CStr::from_ptr(ptr);
//             c_str.to_string_lossy().into_owned()
//         }
//     }
// }
