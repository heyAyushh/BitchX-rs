use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{BitchXError, Result};

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    fn on_load(&mut self) -> Result<()>;
    fn on_unload(&mut self) -> Result<()>;
    fn on_message(&mut self, sender: &str, target: &str, message: &str) -> Option<String>;
}

#[derive(Debug)]
struct LoadedPlugin {
    name: String,
    path: PathBuf,
    _library: Library,
}

#[derive(Debug)]
pub struct PluginManager {
    plugins: HashMap<String, LoadedPlugin>,
    plugin_dir: PathBuf,
}

type PluginCreateFn = unsafe fn() -> *mut dyn Plugin;

impl PluginManager {
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_dir,
        }
    }

    pub fn load(&mut self, path: &Path) -> Result<String> {
        if !path.exists() {
            return Err(BitchXError::Plugin(format!(
                "Plugin file not found: {}",
                path.display()
            )));
        }

        let library = unsafe {
            Library::new(path).map_err(|e| {
                BitchXError::Plugin(format!("Failed to load library {}: {e}", path.display()))
            })?
        };

        let create_fn: Symbol<PluginCreateFn> = unsafe {
            library.get(b"_bitchx_plugin_create").map_err(|e| {
                BitchXError::Plugin(format!(
                    "Plugin {} missing _bitchx_plugin_create symbol: {e}",
                    path.display()
                ))
            })?
        };

        let mut plugin = unsafe { Box::from_raw(create_fn()) };

        let name = plugin.name().to_string();

        if self.plugins.contains_key(&name) {
            return Err(BitchXError::Plugin(format!(
                "Plugin '{}' is already loaded",
                name
            )));
        }

        plugin
            .on_load()
            .map_err(|e| BitchXError::Plugin(format!("Plugin '{}' on_load failed: {e}", name)))?;

        // Intentionally leak the plugin box since we keep the library alive
        std::mem::forget(plugin);

        self.plugins.insert(
            name.clone(),
            LoadedPlugin {
                name: name.clone(),
                path: path.to_path_buf(),
                _library: library,
            },
        );

        Ok(name)
    }

    pub fn unload(&mut self, name: &str) -> Result<()> {
        self.plugins
            .remove(name)
            .ok_or_else(|| BitchXError::Plugin(format!("Plugin '{}' is not loaded", name)))?;
        Ok(())
    }

    pub fn list(&self) -> Vec<(&str, &Path)> {
        self.plugins
            .values()
            .map(|p| (p.name.as_str(), p.path.as_path()))
            .collect()
    }

    pub fn plugin_dir(&self) -> &Path {
        &self.plugin_dir
    }

    pub fn is_loaded(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
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
    }

    #[test]
    fn list_empty_plugins() {
        let pm = PluginManager::new(PathBuf::from("/tmp/plugins"));
        assert!(pm.list().is_empty());
    }

    #[test]
    fn plugin_directory_accessor() {
        let dir = PathBuf::from("/opt/bitchx/plugins");
        let pm = PluginManager::new(dir.clone());
        assert_eq!(pm.plugin_dir(), Path::new("/opt/bitchx/plugins"));
    }

    #[test]
    fn load_nonexistent_plugin_returns_error() {
        let mut pm = PluginManager::new(PathBuf::from("/tmp/plugins"));
        let result = pm.load(Path::new("/nonexistent/plugin.so"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, BitchXError::Plugin(_)));
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
}
