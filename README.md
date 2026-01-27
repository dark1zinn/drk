# drk - A Modular Plugin-Based CLI Tool

`drk` is a dynamic, plugin-based CLI tool written in Rust that demonstrates a modern approach to building extensible command-line applications using a **Dynamic Runtime Architecture**.

## Features

- 🔌 **Plugin Architecture**: Commands and features are loaded at runtime from compiled shared libraries
- 🛡️ **Type-Safe Event System**: Centralized `SystemEvent` enum ensures safe communication between plugins
- 🎯 **Dynamic Command Discovery**: Plugins declare commands via schemas that are automatically integrated into the CLI
- ⚙️ **Lifecycle Management**: Plugins can be enabled/disabled via configuration
- 📝 **Unified Configuration**: Single TOML file for all plugin settings
- 🚀 **FFI-Safe**: Clean separation between the core CLI and plugin implementations

## Architecture

The project is organized as a Cargo workspace with four main components:

```
drk/
├── drk-api/       # The "Contract" - Plugin trait, events, and schemas
├── drk-core/      # The "Engine" - Plugin manager and dynamic loading
├── drk-cli/       # The "Shell" - CLI entry point and command routing
└── plugins/       # Dynamic library plugins
    ├── drk-basic/
    └── drk-logger/
```

## Quick Start

### Building

```bash
cargo build --release
```

### Running

```bash
# Show available commands
./target/release/drk --help

# Execute a command
./target/release/drk greet --name "World"

# Echo a message
./target/release/drk echo --message "Hello from plugins!"
```

### Demo

Run the included demo script to see all features in action:

```bash
./demo.sh
```

## How It Works

### 1. Plugin Loading

At startup, the CLI scans for shared libraries (`.so`, `.dll`, `.dylib`) and dynamically loads them using `libloading`:

```rust
manager.load_plugins_from_dir("./target/debug")?;
```

### 2. Command Discovery

Each plugin implements the `get_commands()` method to declare its commands:

```rust
fn get_commands(&self) -> Vec<PluginCommand> {
    vec![
        PluginCommand {
            name: "greet".to_string(),
            description: "Greet someone by name".to_string(),
            args: vec![
                CommandArg {
                    name: "name".to_string(),
                    description: "The name to greet".to_string(),
                    required: false,
                    arg_type: ArgType::String,
                }
            ],
        }
    ]
}
```

### 3. Dynamic CLI Building

The CLI collects all command schemas and builds a `clap` command tree at runtime:

```rust
let plugin_commands = manager.get_all_plugin_commands();
// Build clap commands from schemas...
```

### 4. Event Routing

When a command is executed, the CLI fires events that plugins can handle:

```rust
// Fired before command execution
SystemEvent::PreCommand { name, args }

// Fired to execute the command
SystemEvent::ExecuteCommand { plugin_name, matches }

// Fired after command execution
SystemEvent::PostCommand { name, success }
```

## Creating a Plugin

### 1. Create a new library crate

```toml
[package]
name = "drk-myplugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
drk-api = { path = "../../drk-api" }
anyhow = "1.0"
```

### 2. Implement the Plugin trait

```rust
use drk_api::{
    declare_plugin, Plugin, PluginMetadata, PluginCommand,
    SystemEvent, Context, CommandArg, ArgType
};
use anyhow::Result;

struct MyPlugin;

impl Plugin for MyPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "myplugin".to_string(),
            version: "0.1.0".to_string(),
            author: "Your Name".to_string(),
            description: "My awesome plugin".to_string(),
            essential: false,
        }
    }

    fn get_commands(&self) -> Vec<PluginCommand> {
        vec![
            PluginCommand {
                name: "mycommand".to_string(),
                description: "Does something cool".to_string(),
                args: vec![
                    CommandArg {
                        name: "arg1".to_string(),
                        description: "An argument".to_string(),
                        required: true,
                        arg_type: ArgType::String,
                    }
                ],
            }
        ]
    }

    fn handle_event(&mut self, event: &SystemEvent, ctx: &mut Context) -> Result<()> {
        match event {
            SystemEvent::ExecuteCommand { plugin_name, matches } 
                if plugin_name == "myplugin" => {
                // Handle your command
                let arg1 = matches.args.get("arg1").unwrap();
                println!("You passed: {}", arg1);
            }
            _ => {}
        }
        Ok(())
    }
}

fn constructor() -> MyPlugin {
    MyPlugin
}

declare_plugin!(MyPlugin, constructor);
```

### 3. Build and use

```bash
cargo build
# The plugin is automatically discovered when you run drk
drk mycommand --arg1 "hello"
```

## Event System

The type-safe event system enables plugins to communicate without tight coupling:

- `SystemEvent::Startup` - Fired when the CLI starts
- `SystemEvent::PreCommand` - Fired before any command execution
- `SystemEvent::ExecuteCommand` - Routes command execution to the owning plugin
- `SystemEvent::PostCommand` - Fired after command execution
- `SystemEvent::Custom` - Custom events between plugins

## Command Schema Types

### ArgType

Supported argument types:
- `String` - Text argument (--name "value")
- `Integer` - Numeric argument (--count 42)
- `Float` - Decimal argument (--ratio 3.14)
- `Boolean` - Flag argument (--verbose)
- `Positional` - Positional argument (value without flag)

### CommandArg

```rust
pub struct CommandArg {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub arg_type: ArgType,
}
```

### PluginCommand

```rust
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub args: Vec<CommandArg>,
}
```

## Configuration

Plugins can be enabled/disabled via configuration:

```toml
[basic]
enabled = true
greeting_prefix = "Hello"

[logger]
enabled = true
```

Essential plugins (marked with `essential: true`) cannot be disabled.

## Project Status

Current implementation status:

- ✅ Workspace and crate structure
- ✅ FFI plugin loading with `libloading`
- ✅ Type-safe event bus (enum-based)
- ✅ Plugin toggling and metadata
- ✅ Dynamic library format for plugins
- ✅ Command schema for dynamic CLI parsing

## Documentation

- [ROADMAP.md](./docs/ROADMAP.md) - Development roadmap and architecture decisions
- [COMMAND_SCHEMA_IMPLEMENTATION.md](./docs/COMMAND_SCHEMA_IMPLEMENTATION.md) - Detailed command schema documentation

## Safety Considerations

### FFI Safety

The plugin system uses `unsafe` code for dynamic loading. Key safety measures:

1. **Double-Drop Prevention**: Plugin instances are dropped before unloading libraries
2. **Symbol Validation**: The `_plugin_create` symbol is verified before calling
3. **Error Handling**: Comprehensive error handling for load failures
4. **Type Safety**: Strong typing via the `Plugin` trait

### Memory Management

- Plugin instances are `Box`ed and properly managed
- Libraries remain loaded while plugins are in use
- Clean shutdown ensures proper resource cleanup

## Performance

- **Lazy Loading**: Plugins are only loaded at startup
- **Zero Runtime Overhead**: Plugin dispatch is direct through trait methods
- **Efficient Routing**: O(1) command lookup via HashMap

## Requirements

- Rust 1.70+ (2021 edition)
- Supported platforms: Linux, macOS, Windows

## Dependencies

Key dependencies:
- `clap` 4.4+ - CLI argument parsing
- `libloading` - Dynamic library loading
- `anyhow` - Error handling
- `serde` / `toml` - Configuration

## Contributing

This is a demonstration project showcasing plugin architecture patterns in Rust. Feel free to:

1. Fork and experiment
2. Add new plugin examples
3. Enhance the event system
4. Improve error handling
5. Add configuration features

## Credits

Built to demonstrate:
- Dynamic plugin architectures in Rust
- FFI-safe interfaces
- Type-safe event systems
- Runtime command discovery
- Modular CLI design patterns

---

**Note**: This is a demonstration project illustrating advanced Rust patterns for building extensible CLI tools. The plugin architecture showcases real-world techniques for creating modular, maintainable command-line applications.