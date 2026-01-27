use drk_api::{Context, Plugin, PluginMetadata, SystemEvent, declare_plugin, style_dim, style_primary, style_success, style_warning};

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
            SystemEvent::Startup => println!("{} System is starting up...", style_dim("[Logger]")),
            SystemEvent::PreCommand { name, .. } => println!("{} About to run: {}", style_dim("[Logger]"), style_primary(name)),
            SystemEvent::PostCommand { name, success } => {
                let status = if *success {
                                    style_success("success")
                                } else {
                                    style_warning("failed")
                                };
                                println!(
                                    "{} Command '{}' completed with status: {}",
                                    style_dim("[Logger]"),
                                    style_primary(name),
                                    status
                                );
            }
            SystemEvent::ExecuteCommand {
                plugin_name,
                matches,
            } => {
                println!(
                                    "{} Executing command '{}' from plugin '{}'",
                                    style_dim("[Logger]"),
                                    style_primary(&matches.command_name),
                                    style_warning(plugin_name)
                                );
            }
            SystemEvent::Custom { source, event, .. } => {
                println!(
                                    "{} Intercepted event '{}' from '{}'",
                                    style_dim("[Logger]"),
                                    style_primary(event),
                                    style_warning(source)
                                );
            }
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
