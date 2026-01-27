use clap::{Arg, ArgAction, Command};
use drk_api::{CommandMatches, SystemEvent};
use drk_core::manager::PluginManager;
use std::collections::HashMap;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let mut manager = PluginManager::new();

    // 1. Define where plugins live
    let plugin_dir = PathBuf::from("./target/debug");

    // 2. Load plugins dynamically
    if plugin_dir.exists() {
        manager.load_plugins_from_dir(plugin_dir)?;
    }

    // 3. Fire Startup event
    manager.fire_event(SystemEvent::Startup);

    // 4. Build the CLI dynamically from plugin commands
    let mut app = Command::new("drk")
        .version("0.1.0")
        .author("drk contributors")
        .about("A modular, plugin-based CLI tool")
        .subcommand_required(false)
        .arg_required_else_help(true);

    // 5. Collect commands from all loaded plugins
    let plugin_commands = manager.get_all_plugin_commands();

    // Map to track which plugin owns which command
    let mut command_to_plugin: HashMap<String, String> = HashMap::new();

    for (plugin_name, commands) in plugin_commands.iter() {
        for cmd in commands {
            // Leak strings to get 'static lifetime for clap
            let cmd_name: &'static str = Box::leak(cmd.name.clone().into_boxed_str());
            let cmd_desc: &'static str = Box::leak(cmd.description.clone().into_boxed_str());

            // Build a clap subcommand from the plugin's command schema
            let mut subcommand = Command::new(cmd_name).about(cmd_desc);

            // Add arguments based on the schema
            for arg in &cmd.args {
                let arg_name: &'static str = Box::leak(arg.name.clone().into_boxed_str());
                let arg_desc: &'static str = Box::leak(arg.description.clone().into_boxed_str());

                let clap_arg = match arg.arg_type {
                    drk_api::ArgType::Positional => Arg::new(arg_name)
                        .help(arg_desc)
                        .required(arg.required)
                        .index(1),
                    drk_api::ArgType::String => Arg::new(arg_name)
                        .long(arg_name)
                        .help(arg_desc)
                        .required(arg.required)
                        .action(ArgAction::Set),
                    drk_api::ArgType::Integer => Arg::new(arg_name)
                        .long(arg_name)
                        .help(arg_desc)
                        .required(arg.required)
                        .value_parser(clap::value_parser!(i64))
                        .action(ArgAction::Set),
                    drk_api::ArgType::Float => Arg::new(arg_name)
                        .long(arg_name)
                        .help(arg_desc)
                        .required(arg.required)
                        .value_parser(clap::value_parser!(f64))
                        .action(ArgAction::Set),
                    drk_api::ArgType::Boolean => Arg::new(arg_name)
                        .long(arg_name)
                        .help(arg_desc)
                        .required(false)
                        .action(ArgAction::SetTrue),
                };

                subcommand = subcommand.arg(clap_arg);
            }

            app = app.subcommand(subcommand);
            command_to_plugin.insert(cmd.name.clone(), plugin_name.clone());
        }
    }

    // 6. Parse command-line arguments
    let matches = app.try_get_matches();

    let matches = match matches {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    // 7. Route to the appropriate plugin
    if let Some((command_name, sub_matches)) = matches.subcommand() {
        // Find which plugin owns this command
        if let Some(plugin_name) = command_to_plugin.get(command_name) {
            // Fire PreCommand event
            let args: Vec<String> = std::env::args().skip(2).collect();
            manager.fire_event(SystemEvent::PreCommand {
                name: command_name.to_string(),
                args: args.clone(),
            });

            // Extract arguments into a simple HashMap
            let mut arg_map = HashMap::new();

            // Get the command schema to know which args to extract
            if let Some(commands) = plugin_commands.get(plugin_name) {
                if let Some(cmd_schema) = commands.iter().find(|c| c.name == command_name) {
                    for arg_def in &cmd_schema.args {
                        match arg_def.arg_type {
                            drk_api::ArgType::String | drk_api::ArgType::Positional => {
                                if let Some(value) = sub_matches.get_one::<String>(&arg_def.name) {
                                    arg_map.insert(arg_def.name.clone(), value.clone());
                                }
                            }
                            drk_api::ArgType::Integer => {
                                if let Some(value) = sub_matches.get_one::<i64>(&arg_def.name) {
                                    arg_map.insert(arg_def.name.clone(), value.to_string());
                                }
                            }
                            drk_api::ArgType::Float => {
                                if let Some(value) = sub_matches.get_one::<f64>(&arg_def.name) {
                                    arg_map.insert(arg_def.name.clone(), value.to_string());
                                }
                            }
                            drk_api::ArgType::Boolean => {
                                if sub_matches.get_flag(&arg_def.name) {
                                    arg_map.insert(arg_def.name.clone(), "true".to_string());
                                }
                            }
                        }
                    }
                }
            }

            // Fire ExecuteCommand event
            let cmd_matches = CommandMatches {
                command_name: command_name.to_string(),
                args: arg_map,
            };

            manager.fire_event(SystemEvent::ExecuteCommand {
                plugin_name: plugin_name.clone(),
                matches: cmd_matches,
            });

            // Fire PostCommand event
            manager.fire_event(SystemEvent::PostCommand {
                name: command_name.to_string(),
                success: true,
            });
        } else {
            eprintln!("Unknown command: {}", command_name);
            std::process::exit(1);
        }
    }

    Ok(())
}
