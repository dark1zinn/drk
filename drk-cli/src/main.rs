use anyhow::{Context as _, Result};
use clap::{Command, ArgMatches};
use drk_core::{Context, EventBus, Plugin, ConfigMap};
use std::fs;
use std::path::PathBuf;

// --- REGISTRY ---
// In a static setup, we manually list built-in plugins here.
fn get_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(drk_basic::BasicPlugin),
        // Add other plugins here e.g. Box::new(drk_http::HttpPlugin),
    ]
}

// --- CONFIG MANAGEMENT ---
fn load_config() -> Result<(ConfigMap, PathBuf)> {
    let dirs = directories::ProjectDirs::from("com", "author", "drk")
        .context("Could not determine config directory")?;
    let config_path = dirs.config_dir().join("config.toml");

    if !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&config_path, "")?;
        return Ok((ConfigMap::new(), config_path));
    }

    let contents = fs::read_to_string(&config_path)?;
    let map: ConfigMap = toml::from_str(&contents).unwrap_or_default();
    Ok((map, config_path))
}

fn save_config(path: &PathBuf, map: &ConfigMap) -> Result<()> {
    let toml_string = toml::to_string(map)?;
    fs::write(path, toml_string)?;
    Ok(())
}

// --- MAIN ---
fn main() -> Result<()> {
    // 1. Initialize Core Systems
    let mut bus = EventBus::new();
    let (mut config, config_path) = load_config()?;
    let plugins = get_plugins();

    // 2. Initialize Context
    let mut ctx = Context {
        bus: &mut bus,
        config: &mut config,
    };

    // 3. Load Plugins (Register Hooks)
    for plugin in &plugins {
        plugin.on_load(&mut ctx)?;
    }

    // 4. Build CLI Interface
    let mut app = Command::new("drk")
        .version("0.1.0")
        .about("The Plugin-based CLI Tool")
        .subcommand_required(true);

    // Map subcommand names to plugin indices
    let mut command_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for (index, plugin) in plugins.iter().enumerate() {
        if let Some(cmd) = plugin.get_command() {
            let name = cmd.get_name().to_string();
            app = app.subcommand(cmd);
            command_map.insert(name, index);
        }
    }

    // 5. Fire Startup Event
    let startup_msg = "Booting up systems...".to_string();
    ctx.bus.publish("app:startup", Some(&startup_msg));

    // 6. Parse Arguments and Dispatch
    let matches = app.get_matches();
    let (sub_name, sub_matches) = matches.subcommand().unwrap();

    if let Some(&idx) = command_map.get(sub_name) {
        let plugin = &plugins[idx];
        
        // Execute plugin logic
        plugin.handle_command(sub_name, sub_matches, &mut ctx)?;
        
        // Save config if modified by plugins
        save_config(&config_path, ctx.config)?;
    }

    Ok(())
}