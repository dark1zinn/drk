use drk_api::{Context, Plugin, PluginMetadata, SystemEvent};
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::Path;
use anyhow::{Context as _, Result};

/// A wrapper around a dynamically loaded plugin.
/// 
/// SAFETY: The `_lib` field MUST be dropped AFTER `instance`. 
/// Rust drops fields in declaration order (top to bottom), so `instance` 
/// is dropped first, then `_lib`. This prevents use-after-free segfaults 
/// where the code is unloaded from memory before the object is destroyed.
struct LoadedPlugin {
    instance: Box<dyn Plugin>,
    _lib: Library,
    #[allow(dead_code)]
    metadata: PluginMetadata,
    enabled: bool,
}

pub struct PluginManager {
    /// Map of Plugin Name -> Loaded Plugin Data
    plugins: HashMap<String, LoadedPlugin>,
    /// Configuration storage (In-memory representation of config.toml)
    config_store: HashMap<String, toml::Value>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            config_store: HashMap::new(),
        }
    }

    /// Recursively scans a directory for shared libraries
    pub fn load_plugins_from_dir<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(());
        }

        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            // Check for library extensions based on OS
            let is_lib = p.extension().map_or(false, |ext| {
                ext == "dll" || ext == "so" || ext == "dylib"
            });

            if is_lib {
                // We use unsafe here because loading arbitrary DLLs is inherently unsafe
                unsafe { 
                    if let Err(e) = self.load_plugin(p) {
                        eprintln!("Failed to load plugin at {:?}: {}", p, e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Loads a single plugin from a path
    unsafe fn load_plugin(&mut self, path: &Path) -> Result<()> {
        // 1. Load the library into memory
        let lib = Library::new(path)
            .with_context(|| format!("Could not open library at {:?}", path))?;

        // 2. Find the entry point symbol
        // This signature MUST match the `_plugin_create` function in `drk-api` macro
        let func: Symbol<unsafe extern "C" fn() -> *mut dyn Plugin> = 
            lib.get(b"_plugin_create")
            .context("Could not find '_plugin_create' symbol. Is this a valid drk plugin?")?;

        // 3. Invoke the creator to get the pointer
        let raw_ptr = func();
        
        // 4. Convert raw pointer back to Box. 
        // We now own this memory.
        let mut instance = Box::from_raw(raw_ptr);

        // 5. Read Metadata
        let metadata = instance.metadata();
        let name = metadata.name.clone();

        // 6. Check if enabled via config
        let enabled = self.is_plugin_enabled(&name, &metadata);

        // 7. Initialize if enabled
        if enabled {
            instance.on_load()?;
        }

        // 8. Store everything. 
        // IMPORTANT: Move `lib` into the struct so it stays alive.
        let loaded = LoadedPlugin {
            instance,
            _lib: lib, 
            metadata: metadata.clone(),
            enabled,
        };

        println!("Loaded Plugin: {} (v{}) [Enabled: {}]", name, metadata.version, enabled);
        self.plugins.insert(name, loaded);

        Ok(())
    }

    fn is_plugin_enabled(&self, name: &str, meta: &PluginMetadata) -> bool {
        if meta.essential {
            return true;
        }
        // Check our in-memory config store
        if let Some(cfg) = self.config_store.get(name) {
             if let Some(val) = cfg.get("enabled") {
                 return val.as_bool().unwrap_or(true);
             }
        }
        true
    }

    /// The Central Event Bus Dispatcher
    /// This replaces the old `EventBus` struct.
    pub fn fire_event(&mut self, event: SystemEvent) {
        // We collect keys first to avoid borrowing `self.plugins` while iterating mutably
        // (Though since we have ownership of the manager here, we can just iterate if careful, 
        // but collecting keys is often safer if plugins try to modify the manager later).
        
        // For this implementation, simple iteration is fine because `handle_event` 
        // takes `&mut Context`, not `&mut PluginManager`.
        
        for (name, plugin) in &mut self.plugins {
            if !plugin.enabled {
                continue;
            }

            // Construct the context to pass into the plugin
            // This exposes the config and a way to emit events (if we had a queue)
            let mut ctx = Context {
                config: &mut self.config_store,
                event_sender: &mut |_evt| {
                    // TODO: In a real system, you push this event to a queue 
                    // and process it after the current loop finishes to avoid recursion depth issues.
                    println!("Plugin {} tried to emit an event (nested events not yet implemented)", name);
                },
            };

            if let Err(e) = plugin.instance.handle_event(&event, &mut ctx) {
                eprintln!("Error in plugin '{}' during event {:?}: {}", name, event, e);
            }
        }
    }
}