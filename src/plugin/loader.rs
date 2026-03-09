use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::{Path, PathBuf};

use crate::error::{BitchYError, Result};

type NameFn = unsafe extern "C" fn() -> *const c_char;
type VersionFn = unsafe extern "C" fn() -> *const c_char;
type DescriptionFn = unsafe extern "C" fn() -> *const c_char;
type InitFn = unsafe extern "C" fn() -> i32;
type CleanupFn = unsafe extern "C" fn() -> i32;
type OnMessageFn =
    unsafe extern "C" fn(*const c_char, *const c_char, *const c_char) -> *const c_char;

#[derive(Debug)]
struct LoadedPlugin {
    name: String,
    version: String,
    description: String,
    path: PathBuf,
    library: Library,
}

#[derive(Debug)]
pub struct PluginManager {
    plugins: HashMap<String, LoadedPlugin>,
    plugin_dir: PathBuf,
}

impl PluginManager {
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_dir,
        }
    }

    /// Load a plugin from a `.so` file.
    ///
    /// The shared library must export the BitchY C ABI symbols:
    /// `bitchy_plugin_name`, `bitchy_plugin_version`, `bitchy_plugin_description`,
    /// `bitchy_plugin_init`, and `bitchy_plugin_cleanup`.
    pub fn load(&mut self, path: &Path) -> Result<String> {
        if !path.exists() {
            return Err(BitchYError::Plugin(format!(
                "Plugin file not found: {}",
                path.display()
            )));
        }

        // SAFETY: We are loading a shared library from a user-specified path.
        // The caller is responsible for ensuring the .so file is a valid BitchY
        // plugin compiled against a compatible ABI. The library must remain loaded
        // for the lifetime of the LoadedPlugin entry.
        let library = unsafe {
            Library::new(path).map_err(|e| {
                BitchYError::Plugin(format!("Failed to load library {}: {e}", path.display()))
            })?
        };

        let name = {
            // SAFETY: The symbol `bitchy_plugin_name` is required to return a
            // pointer to a valid null-terminated C string with static lifetime.
            let func: Symbol<NameFn> = unsafe {
                library.get(b"bitchy_plugin_name").map_err(|e| {
                    BitchYError::Plugin(format!(
                        "Plugin {} missing bitchy_plugin_name: {e}",
                        path.display()
                    ))
                })?
            };
            // SAFETY: The returned pointer must be a valid, null-terminated C string.
            let ptr = unsafe { func() };
            if ptr.is_null() {
                return Err(BitchYError::Plugin(format!(
                    "Plugin {} returned null name",
                    path.display()
                )));
            }
            // SAFETY: We checked for null above; the plugin contract guarantees
            // the pointer is valid and null-terminated.
            unsafe { CStr::from_ptr(ptr) }
                .to_str()
                .map_err(|e| {
                    BitchYError::Plugin(format!(
                        "Plugin {} name is not valid UTF-8: {e}",
                        path.display()
                    ))
                })?
                .to_string()
        };

        let version = {
            // SAFETY: Same contract as bitchy_plugin_name.
            let func: Symbol<VersionFn> = unsafe {
                library.get(b"bitchy_plugin_version").map_err(|e| {
                    BitchYError::Plugin(format!(
                        "Plugin {} missing bitchy_plugin_version: {e}",
                        path.display()
                    ))
                })?
            };
            let ptr = unsafe { func() };
            if ptr.is_null() {
                return Err(BitchYError::Plugin(format!(
                    "Plugin {} returned null version",
                    path.display()
                )));
            }
            // SAFETY: Null check above; plugin contract guarantees validity.
            unsafe { CStr::from_ptr(ptr) }
                .to_str()
                .map_err(|e| {
                    BitchYError::Plugin(format!(
                        "Plugin {} version is not valid UTF-8: {e}",
                        path.display()
                    ))
                })?
                .to_string()
        };

        let description = {
            // SAFETY: Same contract as bitchy_plugin_name.
            let func: Symbol<DescriptionFn> = unsafe {
                library.get(b"bitchy_plugin_description").map_err(|e| {
                    BitchYError::Plugin(format!(
                        "Plugin {} missing bitchy_plugin_description: {e}",
                        path.display()
                    ))
                })?
            };
            let ptr = unsafe { func() };
            if ptr.is_null() {
                return Err(BitchYError::Plugin(format!(
                    "Plugin {} returned null description",
                    path.display()
                )));
            }
            // SAFETY: Null check above; plugin contract guarantees validity.
            unsafe { CStr::from_ptr(ptr) }
                .to_str()
                .map_err(|e| {
                    BitchYError::Plugin(format!(
                        "Plugin {} description is not valid UTF-8: {e}",
                        path.display()
                    ))
                })?
                .to_string()
        };

        if self.plugins.contains_key(&name) {
            return Err(BitchYError::Plugin(format!(
                "Plugin '{}' is already loaded",
                name
            )));
        }

        {
            // SAFETY: bitchy_plugin_init is required to be safe to call once
            // during plugin load. A return value of 0 indicates success.
            let init_fn: Symbol<InitFn> = unsafe {
                library.get(b"bitchy_plugin_init").map_err(|e| {
                    BitchYError::Plugin(format!(
                        "Plugin {} missing bitchy_plugin_init: {e}",
                        path.display()
                    ))
                })?
            };
            let rc = unsafe { init_fn() };
            if rc != 0 {
                return Err(BitchYError::Plugin(format!(
                    "Plugin '{}' init failed with code {rc}",
                    name
                )));
            }
        }

        self.plugins.insert(
            name.clone(),
            LoadedPlugin {
                name: name.clone(),
                version,
                description,
                path: path.to_path_buf(),
                library,
            },
        );

        Ok(name)
    }

    /// Unload a plugin by name.
    ///
    /// Calls `bitchy_plugin_cleanup()` before dropping the library handle.
    pub fn unload(&mut self, name: &str) -> Result<()> {
        let plugin = self
            .plugins
            .remove(name)
            .ok_or_else(|| BitchYError::Plugin(format!("Plugin '{}' is not loaded", name)))?;

        // SAFETY: bitchy_plugin_cleanup is required to be safe to call once
        // during plugin unload. We call it before dropping the library.
        let cleanup_fn: std::result::Result<Symbol<CleanupFn>, _> =
            unsafe { plugin.library.get(b"bitchy_plugin_cleanup") };

        if let Ok(cleanup) = cleanup_fn {
            let rc = unsafe { cleanup() };
            if rc != 0 {
                return Err(BitchYError::Plugin(format!(
                    "Plugin '{}' cleanup failed with code {rc}",
                    name
                )));
            }
        }

        Ok(())
    }

    /// Dispatch a message to all loaded plugins.
    ///
    /// Returns a vec of `(plugin_name, response)` for any plugin that returns
    /// a non-null response string.
    pub fn dispatch_message(
        &self,
        sender: &str,
        target: &str,
        message: &str,
    ) -> Vec<(String, String)> {
        let mut responses = Vec::new();

        let c_sender = match CString::new(sender) {
            Ok(s) => s,
            Err(_) => return responses,
        };
        let c_target = match CString::new(target) {
            Ok(s) => s,
            Err(_) => return responses,
        };
        let c_message = match CString::new(message) {
            Ok(s) => s,
            Err(_) => return responses,
        };

        for plugin in self.plugins.values() {
            // SAFETY: bitchy_plugin_on_message is an optional symbol. If present,
            // the plugin contract guarantees it accepts valid C strings and returns
            // either null or a pointer to a CString allocated with CString::into_raw.
            let on_msg: std::result::Result<Symbol<OnMessageFn>, _> =
                unsafe { plugin.library.get(b"bitchy_plugin_on_message") };

            if let Ok(func) = on_msg {
                let ptr = unsafe { func(c_sender.as_ptr(), c_target.as_ptr(), c_message.as_ptr()) };

                if !ptr.is_null() {
                    // SAFETY: The plugin contract says non-null returns are
                    // pointers from CString::into_raw(), so we must retake
                    // ownership to free the memory.
                    let response = unsafe { CString::from_raw(ptr as *mut c_char) };
                    if let Ok(s) = response.into_string() {
                        responses.push((plugin.name.clone(), s));
                    }
                }
            }
        }

        responses
    }

    /// List loaded plugins as `(name, version, description, path)`.
    pub fn list(&self) -> Vec<(&str, &str, &str, &Path)> {
        self.plugins
            .values()
            .map(|p| {
                (
                    p.name.as_str(),
                    p.version.as_str(),
                    p.description.as_str(),
                    p.path.as_path(),
                )
            })
            .collect()
    }

    pub fn plugin_dir(&self) -> &Path {
        &self.plugin_dir
    }

    pub fn is_loaded(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    pub fn count(&self) -> usize {
        self.plugins.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn create_plugin_manager() {
        let dir = PathBuf::from("/tmp/plugins");
        let pm = PluginManager::new(dir.clone());
        assert!(pm.list().is_empty());
        assert_eq!(pm.plugin_dir(), dir);
        assert_eq!(pm.count(), 0);
    }

    #[test]
    fn list_empty_plugins() {
        let pm = PluginManager::new(PathBuf::from("/tmp/plugins"));
        assert!(pm.list().is_empty());
    }

    #[test]
    fn plugin_directory_accessor() {
        let dir = PathBuf::from("/opt/bitchy/plugins");
        let pm = PluginManager::new(dir.clone());
        assert_eq!(pm.plugin_dir(), Path::new("/opt/bitchy/plugins"));
    }

    #[test]
    fn load_nonexistent_plugin_returns_error() {
        let mut pm = PluginManager::new(PathBuf::from("/tmp/plugins"));
        let result = pm.load(Path::new("/nonexistent/plugin.so"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, BitchYError::Plugin(_)));
    }

    #[test]
    fn is_loaded_returns_false_for_unknown_plugin() {
        let pm = PluginManager::new(PathBuf::from("/tmp/plugins"));
        assert!(!pm.is_loaded("nonexistent"));
    }

    #[test]
    fn unload_nonexistent_plugin_returns_error() {
        let mut pm = PluginManager::new(PathBuf::from("/tmp/plugins"));
        let result = pm.unload("nothing");
        assert!(result.is_err());
    }

    #[test]
    fn dispatch_message_with_no_plugins_returns_empty() {
        let pm = PluginManager::new(PathBuf::from("/tmp/plugins"));
        let responses = pm.dispatch_message("nick", "#channel", "hello");
        assert!(responses.is_empty());
    }

    #[test]
    fn count_returns_zero_initially() {
        let pm = PluginManager::new(PathBuf::from("/tmp/plugins"));
        assert_eq!(pm.count(), 0);
    }

    #[test]
    fn plugin_dir_matches_constructor() {
        let dir = PathBuf::from("/home/user/.bitchy/plugins");
        let pm = PluginManager::new(dir.clone());
        assert_eq!(pm.plugin_dir(), dir.as_path());
    }
}
