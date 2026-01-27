use drk_core::manager::PluginManager;
use drk_api::SystemEvent;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let mut manager = PluginManager::new();

    // 1. Define where plugins live (e.g., "./plugins_bin")
    let plugin_dir = PathBuf::from("./target/debug"); // Just for testing locally
    
    // 2. Load them dynamically
    // In a real run, you'd point this to ~/.drk/plugins or similar
    if plugin_dir.exists() {
        manager.load_plugins_from_dir(plugin_dir)?;
    }

    // 3. Fire Startup
    manager.fire_event(SystemEvent::Startup);

    // 4. Simulate running a command
    manager.fire_event(SystemEvent::PreCommand { 
        name: "status".into(), 
        args: vec![] 
    });

    Ok(())
}