use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn bitchy_plugin_name() -> *const c_char {
    c"hello".as_ptr()
}

#[no_mangle]
pub extern "C" fn bitchy_plugin_version() -> *const c_char {
    c"1.0.0".as_ptr()
}

#[no_mangle]
pub extern "C" fn bitchy_plugin_description() -> *const c_char {
    c"Responds to !hello with a greeting".as_ptr()
}

#[no_mangle]
pub extern "C" fn bitchy_plugin_init() -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn bitchy_plugin_cleanup() -> i32 {
    0
}

/// # Safety
///
/// `sender`, `target`, and `message` must be valid, non-null, null-terminated
/// C strings, or null. A null `message` returns null.
#[no_mangle]
pub unsafe extern "C" fn bitchy_plugin_on_message(
    _sender: *const c_char,
    _target: *const c_char,
    message: *const c_char,
) -> *const c_char {
    if message.is_null() {
        return std::ptr::null();
    }

    // SAFETY: Caller guarantees `message` is a valid, null-terminated C string.
    let msg = unsafe { CStr::from_ptr(message) };
    let msg_str = match msg.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null(),
    };

    if msg_str.starts_with("!hello") {
        let response =
            CString::new("Hello from the BitchY hello plugin!").expect("CString::new failed");
        response.into_raw() as *const c_char
    } else {
        std::ptr::null()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_name() {
        let name = bitchy_plugin_name();
        // SAFETY: `bitchy_plugin_name` returns a pointer to a static C string literal.
        let name_str = unsafe { CStr::from_ptr(name) }.to_str().unwrap();
        assert_eq!(name_str, "hello");
    }

    #[test]
    fn test_plugin_version() {
        let version = bitchy_plugin_version();
        // SAFETY: `bitchy_plugin_version` returns a pointer to a static C string literal.
        let version_str = unsafe { CStr::from_ptr(version) }.to_str().unwrap();
        assert_eq!(version_str, "1.0.0");
    }

    #[test]
    fn test_plugin_description() {
        let desc = bitchy_plugin_description();
        // SAFETY: `bitchy_plugin_description` returns a pointer to a static C string literal.
        let desc_str = unsafe { CStr::from_ptr(desc) }.to_str().unwrap();
        assert_eq!(desc_str, "Responds to !hello with a greeting");
    }

    #[test]
    fn test_plugin_init() {
        assert_eq!(bitchy_plugin_init(), 0);
    }

    #[test]
    fn test_plugin_cleanup() {
        assert_eq!(bitchy_plugin_cleanup(), 0);
    }

    #[test]
    fn test_on_message_hello() {
        let sender = CString::new("testuser").unwrap();
        let target = CString::new("#test").unwrap();
        let msg = CString::new("!hello").unwrap();
        // SAFETY: All pointers are valid CStrings produced above.
        let result =
            unsafe { bitchy_plugin_on_message(sender.as_ptr(), target.as_ptr(), msg.as_ptr()) };
        assert!(!result.is_null());
        // SAFETY: Non-null result is a heap-allocated CString per the plugin contract.
        let response = unsafe { CStr::from_ptr(result) }.to_str().unwrap();
        assert!(response.contains("Hello"));
        // SAFETY: The pointer was produced by `CString::into_raw`; we reclaim ownership.
        unsafe {
            drop(CString::from_raw(result as *mut c_char));
        }
    }

    #[test]
    fn test_on_message_hello_with_args() {
        let sender = CString::new("testuser").unwrap();
        let target = CString::new("#test").unwrap();
        let msg = CString::new("!hello world").unwrap();
        // SAFETY: All pointers are valid CStrings produced above.
        let result =
            unsafe { bitchy_plugin_on_message(sender.as_ptr(), target.as_ptr(), msg.as_ptr()) };
        assert!(!result.is_null());
        // SAFETY: Non-null result is a heap-allocated CString per the plugin contract.
        let response = unsafe { CStr::from_ptr(result) }.to_str().unwrap();
        assert!(response.contains("Hello"));
        // SAFETY: The pointer was produced by `CString::into_raw`; we reclaim ownership.
        unsafe {
            drop(CString::from_raw(result as *mut c_char));
        }
    }

    #[test]
    fn test_on_message_no_match() {
        let sender = CString::new("testuser").unwrap();
        let target = CString::new("#test").unwrap();
        let msg = CString::new("regular message").unwrap();
        // SAFETY: All pointers are valid CStrings produced above.
        let result =
            unsafe { bitchy_plugin_on_message(sender.as_ptr(), target.as_ptr(), msg.as_ptr()) };
        assert!(result.is_null());
    }

    #[test]
    fn test_on_message_null_message() {
        let sender = CString::new("testuser").unwrap();
        let target = CString::new("#test").unwrap();
        // SAFETY: Null message is explicitly handled by returning null.
        let result =
            unsafe { bitchy_plugin_on_message(sender.as_ptr(), target.as_ptr(), std::ptr::null()) };
        assert!(result.is_null());
    }
}
