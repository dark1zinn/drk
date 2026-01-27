use drk_api::{declare_plugin, Context, Plugin, PluginMetadata, SystemEvent};
use anyhow::Result;

// 1. Define the Plugin Struct
struct BasicPlugin;

// 2. Implement the Trait
impl Plugin for BasicPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "basic".to_string(),
            version: "0.1.0".to_string(),
            author: "You".to_string(),
            description: "A basic plugin that greets".to_string(),
            essential: false,
        }
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

            // Hook into command execution
            // NOTE: Since we haven't built the Schema yet, the CLI doesn't technically
            // know "greet" is a valid command. We are checking the raw args here 
            // as a temporary integration test.
            SystemEvent::PreCommand { name, args } => {
                if name == "greet" {
                    let user_name = args.first().map(|s| s.as_str()).unwrap_or("World");
                    
                    // Access config safely
                    let mut prefix = "Hello".to_string();
                    if let Some(cfg) = ctx.config.get("basic") {
                        if let Some(val) = cfg.get("greeting_prefix") {
                             prefix = val.as_str().unwrap_or("Hello").to_string();
                        }
                    }

                    println!("{} {}!", prefix, user_name);

                    // We can also fire a custom event back to the system
                    (ctx.event_sender)(SystemEvent::Custom { 
                        source: "basic".into(), 
                        event: "greeted".into(), 
                        payload: None 
                    });
                }
            }

            _ => {}
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