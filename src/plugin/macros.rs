/// Macro to declare a BitchY plugin with the required C ABI exports.
///
/// # Usage
///
/// ```ignore
/// bitchy::declare_bitchy_plugin!(
///     "my_plugin",
///     "1.0.0",
///     "A sample plugin",
///     |sender, target, message| {
///         if message.contains("hello") {
///             Some("Hello back!".to_string())
///         } else {
///             None
///         }
///     }
/// );
/// ```
///
/// The macro generates all required `extern "C"` functions for the BitchY
/// plugin C ABI: `bitchy_plugin_name`, `bitchy_plugin_version`,
/// `bitchy_plugin_description`, `bitchy_plugin_init`, `bitchy_plugin_cleanup`,
/// and `bitchy_plugin_on_message`.
#[macro_export]
macro_rules! declare_bitchy_plugin {
    ($name:expr, $version:expr, $description:expr, $on_message:expr) => {
        use std::ffi::{CStr, CString};
        use std::os::raw::c_char;

        #[no_mangle]
        pub extern "C" fn bitchy_plugin_name() -> *const c_char {
            concat!($name, "\0").as_ptr() as *const c_char
        }

        #[no_mangle]
        pub extern "C" fn bitchy_plugin_version() -> *const c_char {
            concat!($version, "\0").as_ptr() as *const c_char
        }

        #[no_mangle]
        pub extern "C" fn bitchy_plugin_description() -> *const c_char {
            concat!($description, "\0").as_ptr() as *const c_char
        }

        #[no_mangle]
        pub extern "C" fn bitchy_plugin_init() -> i32 {
            0
        }

        #[no_mangle]
        pub extern "C" fn bitchy_plugin_cleanup() -> i32 {
            0
        }

        #[no_mangle]
        pub extern "C" fn bitchy_plugin_on_message(
            sender: *const c_char,
            target: *const c_char,
            message: *const c_char,
        ) -> *const c_char {
            // SAFETY: The host (PluginManager) guarantees that sender, target,
            // and message are valid, non-null, null-terminated C strings.
            let sender = unsafe { CStr::from_ptr(sender) }.to_str().unwrap_or("");
            let target = unsafe { CStr::from_ptr(target) }.to_str().unwrap_or("");
            let message = unsafe { CStr::from_ptr(message) }.to_str().unwrap_or("");

            let handler: fn(&str, &str, &str) -> Option<String> = $on_message;
            match handler(sender, target, message) {
                Some(response) => match CString::new(response) {
                    Ok(c) => c.into_raw(),
                    Err(_) => std::ptr::null(),
                },
                None => std::ptr::null(),
            }
        }
    };
}
