use anyhow::Result;
use drk_api::{
    declare_plugin, ArgType, CommandArg, CommandMatches, Context, Plugin, PluginCommand,
    PluginMetadata, SystemEvent,
    style_primary, style_success, icon_success, icon_info, icon_error, style_error
};

// 1. Define the Plugin Struct
struct BasicPlugin;

// 2. Implement the Trait
impl Plugin for BasicPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "basic".to_string(),
            version: "0.1.0".to_string(),
            author: "You".to_string(),
            description: "A basic plugin with greet and echo commands".to_string(),
            essential: false,
        }
    }

    fn get_commands(&self) -> Vec<PluginCommand> {
        vec![
            // Greet command
            PluginCommand {
                name: "greet".to_string(),
                description: "Greet someone by name".to_string(),
                args: vec![CommandArg {
                    name: "name".to_string(),
                    description: "The name to greet".to_string(),
                    required: false,
                    arg_type: ArgType::String,
                }],
            },
            // Echo command
            PluginCommand {
                name: "echo".to_string(),
                description: "Echo back a message".to_string(),
                args: vec![CommandArg {
                    name: "message".to_string(),
                    description: "The message to echo".to_string(),
                    required: true,
                    arg_type: ArgType::String,
                }],
            },
        ]
    }

    fn on_load(&mut self) -> Result<()> {
        println!("[BasicPlugin] Loaded and ready!");
        Ok(())
    }

    fn handle_event(&mut self, event: &SystemEvent, ctx: &mut Context) -> Result<()> {
        match event {
            // Hook into the application startup
            SystemEvent::Startup => {
                println!("[BasicPlugin] I see the app is starting.");
            }

            // Handle command execution
            SystemEvent::ExecuteCommand {
                plugin_name,
                matches,
            } => {
                // Only handle commands meant for this plugin
                if plugin_name == "basic" {
                    self.execute_command(matches, ctx)?;
                }
            }

            // Legacy hook for PreCommand (for backward compatibility)
            SystemEvent::PreCommand { name, .. } => {
                println!("[BasicPlugin] PreCommand hook: {}", name);
            }

            _ => {}
        }
        Ok(())
    }
}

impl BasicPlugin {
    fn execute_command(&self, matches: &CommandMatches, ctx: &mut Context) -> Result<()> {
        match matches.command_name.as_str() {
            "greet" => {
                let name = matches
                    .args
                    .get("name")
                    .map(|s| s.as_str())
                    .unwrap_or("World");

                // Access config safely for greeting prefix
                let mut prefix = "Hello".to_string();
                if let Some(cfg) = ctx.config.get("basic") {
                    if let Some(val) = cfg.get("greeting_prefix") {
                        prefix = val.as_str().unwrap_or("Hello").to_string();
                    }
                }

                println!("{} {} {}{}", style_success(icon_success()), style_success(&prefix), style_primary(name), style_success("!"));

                // Fire a custom event back to the system
                (ctx.event_sender)(SystemEvent::Custom {
                    source: "basic".into(),
                    event: "greeted".into(),
                    payload: None,
                });
            }

            "echo" => {
                if let Some(message) = matches.args.get("message") {
                    println!("{} {}{}", style_success(icon_info()), style_primary(message), style_success("!"));
                } else {
                    eprintln!("{}: {}", style_error(icon_error()), style_error("message argument is required!"));
                }
            }

            _ => {
                eprintln!("{} {} {}", style_error(icon_error()), style_error("Unknown command:"), style_primary(&matches.command_name));
            }
        }
        Ok(())
    }
}

// 3. Define the Constructor
fn constructor() -> BasicPlugin {
    BasicPlugin
}

// 4. Export the symbols so the Manager can find them
declare_plugin!(BasicPlugin, constructor);
