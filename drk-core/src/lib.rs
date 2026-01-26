use anyhow::Result;
use clap::Command;
use downcast_rs::{impl_downcast, Downcast};
use std::any::Any;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// --- METADATA ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
}

// --- CONFIGURATION ---
// The main config map. Keys are plugin names, values are arbitrary TOML tables.
pub type ConfigMap = HashMap<String, toml::Table>;

// --- EVENT SYSTEM ---
pub struct EventBus {
    // Map of Event Name -> List of Handlers
    listeners: HashMap<String, Vec<Box<dyn EventHandler>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self { listeners: HashMap::new() }
    }

    pub fn subscribe<F>(&mut self, event_name: &str, handler: F)
    where
        F: EventHandler + 'static,
    {
        self.listeners
            .entry(event_name.to_string())
            .or_default()
            .push(Box::new(handler));
    }

    pub fn publish(&self, event_name: &str, payload: Option<&dyn Any>) {
        if let Some(handlers) = self.listeners.get(event_name) {
            for handler in handlers {
                handler.handle(payload);
            }
        }
    }
}

// Trait for functions that handle events
pub trait EventHandler: Send + Sync {
    fn handle(&self, payload: Option<&dyn Any>);
}

impl<F> EventHandler for F
where
    F: Fn(Option<&dyn Any>) + Send + Sync,
{
    fn handle(&self, payload: Option<&dyn Any>) {
        self(payload)
    }
}

// --- CONTEXT ---
// Passed to plugins so they can interact with the system
pub struct Context<'a> {
    pub bus: &'a mut EventBus,
    pub config: &'a mut ConfigMap,
}

// --- PLUGIN TRAIT ---
pub trait Plugin: Downcast + Send + Sync {
    /// Returns basic info about the plugin
    fn metadata(&self) -> PluginMetadata;

    /// Called on initialization. Use this to register hooks/events.
    fn on_load(&self, _ctx: &mut Context) -> Result<()> {
        Ok(())
    }

    /// If the plugin adds a CLI command, define it here.
    fn get_command(&self) -> Option<Command> {
        None
    }

    /// Called when the CLI executes a command matching this plugin.
    fn handle_command(
        &self,
        _command_name: &str,
        _args: &clap::ArgMatches,
        _ctx: &mut Context
    ) -> Result<()> {
        Ok(())
    }
}
impl_downcast!(Plugin);