use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;

pub mod styling;

pub use styling::{
    icon_error,
    icon_info,
    icon_success,
    icon_warning,
    // TODO: icon_question,
    style_dim,
    style_error,
    style_primary,
    style_success,
    style_warning,
};

// --- 1. COMMAND SCHEMA SYSTEM ---
// A serializable representation of a CLI command that can cross FFI boundaries.

/// Represents an argument for a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandArg {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub arg_type: ArgType,
}

/// The type of argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArgType {
    String,
    Integer,
    Float,
    Boolean,
    /// A positional argument
    Positional,
}

/// Represents a command that a plugin can handle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub args: Vec<CommandArg>,
}

/// Parsed arguments from a command execution
#[derive(Debug, Clone)]
pub struct CommandMatches {
    pub command_name: String,
    pub args: HashMap<String, String>,
}

// --- 2. TYPE-SAFE EVENT SYSTEM ---
// Instead of just Strings, we use an Enum to strictly define Core events.
// Plugins can use `Custom` to pass data, but they should document their data payload.

#[derive(Debug, Clone)]
pub enum SystemEvent {
    /// Fired when CLI starts.
    Startup,
    /// Fired before a command runs.
    PreCommand { name: String, args: Vec<String> },
    /// Fired after a command runs.
    PostCommand { name: String, success: bool },
    /// Fired when a command should be executed by its owning plugin.
    ExecuteCommand {
        plugin_name: String,
        matches: CommandMatches,
    },
    /// A custom hook from another plugin.
    /// Plugins should document: "I fire 'http:request' with payload 'HttpRequest'"
    Custom {
        source: String,
        event: String,
        payload: Option<std::sync::Arc<dyn Any + Send + Sync>>,
    },
}

// --- 3. PLUGIN METADATA ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    /// Is this plugin critical? If true, it cannot be disabled.
    pub essential: bool,
}

// --- 4. CONTEXT ---
// Passed to every plugin function.
pub struct Context<'a> {
    // We hide the config map behind a safe accessor to prevent conflicts
    pub config: &'a mut HashMap<String, toml::Value>,
    // A way to fire events back to the manager
    pub event_sender: &'a mut dyn FnMut(SystemEvent),
}

// --- 5. THE PLUGIN TRAIT ---
// All dynamic plugins must implement this.
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;

    /// Returns the commands this plugin provides.
    /// This is called during CLI initialization to build the command tree.
    fn get_commands(&self) -> Vec<PluginCommand> {
        Vec::new()
    }

    fn on_load(&mut self) -> Result<()> {
        Ok(())
    }

    fn on_unload(&mut self) -> Result<()> {
        Ok(())
    }

    // The handler now takes the strict SystemEvent enum
    fn handle_event(&mut self, event: &SystemEvent, ctx: &mut Context) -> Result<()>;
}

// --- 6. FFI MACRO ---
// Plugins will use this macro to export themselves safely.
#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty, $constructor:path) => {
        #[no_mangle]
        pub extern "C" fn _plugin_create() -> *mut dyn $crate::Plugin {
            // Create the plugin and leak it into a raw pointer for the CLI to take
            let constructor: fn() -> $plugin_type = $constructor;
            let object = constructor();
            let boxed: Box<dyn $crate::Plugin> = Box::new(object);
            Box::into_raw(boxed)
        }
    };
}
