use drk_core::{Plugin, PluginMetadata, Context};
use clap::{Command, Arg};
use anyhow::Result;

pub struct BasicPlugin;

impl Plugin for BasicPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "basic".to_string(),
            version: "0.1.0".to_string(),
            author: "You".to_string(),
            description: "A basic plugin that greets".to_string(),
        }
    }

    fn on_load(&self, ctx: &mut Context) -> Result<()> {
        // Register a hook: Listen for "app:startup"
        ctx.bus.subscribe("app:startup", |payload| {
            if let Some(msg) = payload.and_then(|p| p.downcast_ref::<String>()) {
                println!("[BasicPlugin Hook]: App started! Message: {}", msg);
            }
        });
        Ok(())
    }

    fn get_command(&self) -> Option<Command> {
        Some(Command::new("greet")
            .about("Prints a greeting")
            .arg(Arg::new("name").short('n').help("Name to greet")))
    }

    fn handle_command(
        &self,
        command_name: &str,
        args: &clap::ArgMatches,
        ctx: &mut Context
    ) -> Result<()> {
        if command_name == "greet" {
            let name = args.get_one::<String>("name").map(|s| s.as_str()).unwrap_or("World");
            
            // Check config if it exists
            let mut prefix = "Hello".to_string();
            if let Some(cfg) = ctx.config.get("basic") {
                if let Some(val) = cfg.get("greeting_prefix") {
                     prefix = val.as_str().unwrap_or("Hello").to_string();
                }
            }

            println!("{} {}!", prefix, name);
            
            // Publish an event that other plugins could hook into
            ctx.bus.publish("basic:greeted", Some(&name.to_string()));
        }
        Ok(())
    }
}