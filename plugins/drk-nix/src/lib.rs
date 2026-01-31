use drk_api::{
    ArgType, CommandArg, CommandMatches, Context, Plugin, PluginCommand, PluginMetadata, SystemEvent, declare_plugin, icon_error, icon_info, icon_warning, style_error, style_primary, style_warning
};
use serde::Deserialize;

struct NixPlugin;

#[derive(Debug, Deserialize, PartialEq)]
struct Template {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GithubItem {
    name: String,
    #[serde(rename = "type")]
    item_type: String,
}

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
        #[allow(unused_variables)] ctx: &mut Context,
    ) -> anyhow::Result<()> {
        match matches.command_name.as_str() {
            "nix" => {
                let template: Template = matches
                    .args
                    .get("template")
                    .map(|s| Template { name: s.to_string() })
                    .unwrap_or(Template { name: "empty".to_string() });
                
                let gh_templates = self.fetch_gh_templates()?;
                
                if !gh_templates.contains(&template) {
                    println!("{} {}", style_warning(icon_warning()), style_warning("Template not found!"));
                    println!("{} {}", style_primary(icon_info()), style_primary("You may wanne check out the available templates at https://github.com/the-nix-way/dev-templates"));
                    return Ok(())
                }
                
                println!(
                    "{} Initializing nix flake dev environment template: {}",
                    style_warning(icon_info()),
                    style_primary(&template.name)
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

    /// Fetches the list of directories from a GitHub repository
    /// # Returns
    /// * `Result<Vec<Template>, anyhow::Error>` - List of directory names or error
    fn fetch_gh_templates(&self) -> Result<Vec<Template>, anyhow::Error> {
        // GH api URL pointing to flake templates provided by the-nix-way/dev-templates
        let tnw_templates_url =
            format!("https://api.github.com/repos/the-nix-way/dev-templates/contents");
        
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&tnw_templates_url)
            .header("User-Agent", "drk-nix-plugin")
            .send()?;
        
        if !response.status().is_success() {
            anyhow::bail!(
                "{} {}{}",
                style_error(icon_error()),
                style_warning("Failed to fetch templates from Github.\n"),
                style_error(response.status().as_str())
            );
        }
        
        let items: Vec<GithubItem> = response.json()?;
        
        let templates: Vec<Template> = items
            .into_iter()
            .filter(|item| item.item_type == "dir" && !item.name.starts_with("."))
            .map(|item| Template { name: item.name })
            .collect();
        
        Ok(templates)
    }
}

// Helper to create the plugin instance
fn constructor() -> NixPlugin {
    NixPlugin
}

declare_plugin!(NixPlugin, constructor);
