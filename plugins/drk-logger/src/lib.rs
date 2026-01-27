use drk_api::{Plugin, PluginMetadata, SystemEvent, Context, declare_plugin};

struct LoggerPlugin;

impl Plugin for LoggerPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "logger".to_string(),
            version: "0.0.1".to_string(),
            author: "dark1zinn".to_string(),
            description: "Logs events to console".to_string(),
            essential: false,
        }
    }

    fn handle_event(&mut self, event: &SystemEvent, _ctx: &mut Context) -> anyhow::Result<()> {
        match event {
            SystemEvent::Startup => println!("[Logger] System is starting up..."),
            SystemEvent::PreCommand { name, .. } => println!("[Logger] About to run: {}", name),
            SystemEvent::Custom { source, event, .. } => {
                println!("[Logger] Intercepted event '{}' from '{}'", event, source);
            }
            _ => {}
        }
        Ok(())
    }
}

// Helper to create the plugin instance
fn constructor() -> LoggerPlugin {
    LoggerPlugin
}

// Export the symbols
declare_plugin!(LoggerPlugin, constructor);