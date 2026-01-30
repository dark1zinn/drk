use drk_api::{
    declare_plugin, icon_info, icon_warning, style_primary,
    style_warning, ArgType, CommandArg, CommandMatches, Context, Plugin, PluginCommand,
    PluginMetadata, SystemEvent,
};

struct NixPlugin;

impl Plugin for NixPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "Nix".to_string(),
            description: "Nix plugin".to_string(),
            version: "0.1.0".to_string(),
            author: "dark1zinn".to_string(),
            essential: false,
        }
    }

    fn get_commands(&self) -> Vec<PluginCommand> {
        vec![
            PluginCommand {
                name: "nix".to_string(),
                description: "Nix command".to_string(),
                args: vec![CommandArg {
                    name: "template".to_string(),
                    description: "Initialize a nix flake dev environment template, pass the name of the template from the-nix-way/dev-templates".to_string(),
                    required: true,
                    arg_type: ArgType::String,
                }],
            },
        ]
    }

    fn on_load(&mut self) -> anyhow::Result<()> {
        println!("{}", style_primary("[NixPlugin] Loaded!"));
        Ok(())
    }

    fn on_unload(&mut self) -> anyhow::Result<()> {
        println!("{}", style_primary("[NixPlugin] Unloaded!"));
        Ok(())
    }

    fn handle_event(&mut self, event: &SystemEvent, ctx: &mut Context) -> anyhow::Result<()> {
        match event {
            SystemEvent::ExecuteCommand {
                plugin_name,
                matches,
            } => {
                // Only handle commands meant for this plugin
                if plugin_name == "Nix" {
                    self.execute_command(matches, ctx)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl NixPlugin {
    fn execute_command(
        &mut self,
        matches: &CommandMatches,
        #[allow(unused_variables)]
        ctx: &mut Context,
    ) -> anyhow::Result<()> {
        match matches.command_name.as_str() {
            "nix" => {
                let template = matches.args.get("template").map(|s| s.as_str()).unwrap_or("empty");
                println!(
                    "{} Initializing nix flake dev environment template: {}",
                    style_warning(icon_info()),
                    style_primary(&template)
                );
            }
            _ => println!(
                "{} Unknown command: {}",
                style_warning(icon_warning()),
                style_primary(&matches.command_name)
            ),
        }
        Ok(())
    }
}

// Helper to create the plugin instance
fn constructor() -> NixPlugin {
    NixPlugin
}

declare_plugin!(NixPlugin, constructor);
