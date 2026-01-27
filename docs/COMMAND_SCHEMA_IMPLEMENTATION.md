# Command Schema Implementation Summary

## Overview

This document describes the implementation of the **Command Schema System** for the `drk` CLI tool, which enables plugins to dynamically register commands that are automatically integrated into the main CLI interface.

## Problem Statement

Previously, the CLI binary didn't know what commands a plugin provided until after the plugin was loaded. Additionally, we couldn't pass `clap::Command` objects across the FFI boundary safely because `clap` does not have a stable ABI.

## Solution

We implemented a **serializable Command Schema** that allows plugins to describe their commands in a simple, FFI-safe format. The CLI then uses these schemas to dynamically build the `clap` command tree at runtime.

## Implementation Details

### 1. Command Schema Types (drk-api)

Added the following types to `drk-api/src/lib.rs`:

#### `CommandArg`
```rust
pub struct CommandArg {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub arg_type: ArgType,
}
```

Represents an argument that a command can accept.

#### `ArgType`
```rust
pub enum ArgType {
    String,
    Integer,
    Float,
    Boolean,
    Positional,
}
```

Defines the type of argument for proper parsing and validation.

#### `PluginCommand`
```rust
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub args: Vec<CommandArg>,
}
```

The main schema structure that describes a command.

#### `CommandMatches`
```rust
pub struct CommandMatches {
    pub command_name: String,
    pub args: HashMap<String, String>,
}
```

Contains parsed arguments that are passed to plugins when a command is executed.

### 2. Event System Updates

Added a new event type to `SystemEvent`:

```rust
ExecuteCommand {
    plugin_name: String,
    matches: CommandMatches,
}
```

This event is fired when the CLI routes a command to its owning plugin.

### 3. Plugin Trait Extension

Added a new method to the `Plugin` trait:

```rust
fn get_commands(&self) -> Vec<PluginCommand> {
    Vec::new()
}
```

Plugins implement this method to declare the commands they provide.

### 4. Plugin Manager Extension (drk-core)

Added `get_all_plugin_commands()` method:

```rust
pub fn get_all_plugin_commands(&self) -> HashMap<String, Vec<PluginCommand>>
```

This method collects command schemas from all enabled plugins, returning a map of plugin names to their command lists.

### 5. Dynamic CLI Building (drk-cli)

The CLI now follows this workflow:

1. **Load Plugins**: Dynamically load all plugin libraries from the plugin directory
2. **Collect Schemas**: Call `get_all_plugin_commands()` to gather command definitions
3. **Build Clap Tree**: Iterate through schemas and construct `clap::Command` objects
4. **Parse Arguments**: Use clap to parse command-line arguments
5. **Route to Plugin**: Identify which plugin owns the command and fire `ExecuteCommand` event
6. **Execute**: The plugin receives the event and executes the command logic

#### Key Technical Details

- **Static Lifetime Requirement**: Clap requires `'static` lifetime strings. We use `Box::leak()` to convert owned strings to static references. This is acceptable in a CLI application that builds the command tree once at startup.
- **Type Mapping**: Each `ArgType` is mapped to the appropriate clap `Arg` configuration with proper value parsers.
- **Command Routing**: A `HashMap<String, String>` tracks which plugin owns each command name.

### 6. Plugin Implementation Example

The `drk-basic` plugin demonstrates the implementation:

```rust
fn get_commands(&self) -> Vec<PluginCommand> {
    vec![
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

fn handle_event(&mut self, event: &SystemEvent, ctx: &mut Context) -> Result<()> {
    match event {
        SystemEvent::ExecuteCommand { plugin_name, matches } => {
            if plugin_name == "basic" {
                self.execute_command(matches, ctx)?;
            }
        }
        // ... other events
    }
    Ok(())
}
```

## Usage Examples

### Display Help
```bash
$ drk --help
A modular, plugin-based CLI tool

Usage: drk [COMMAND]

Commands:
  greet  Greet someone by name
  echo   Echo back a message
  help   Print this message or the help of the given subcommand(s)
```

### Execute Commands
```bash
$ drk greet --name Alice
Hello Alice!

$ drk greet
Hello World!

$ drk echo --message "Hello from plugins"
Hello from plugins
```

## Architecture Benefits

1. **Type Safety**: All command definitions are strongly typed and validated at compile time
2. **FFI Safe**: The schema uses only simple, serializable types that cross FFI boundaries safely
3. **Discoverable**: Commands are automatically discovered and integrated into help text
4. **Decoupled**: Plugins don't need to know about clap or CLI implementation details
5. **Extensible**: New argument types can be added to `ArgType` enum as needed

## Event Flow

```
User Input
    ↓
Clap Parser (built from schemas)
    ↓
CLI identifies owning plugin
    ↓
Fire PreCommand event (all plugins notified)
    ↓
Fire ExecuteCommand event (target plugin executes)
    ↓
Fire PostCommand event (all plugins notified)
```

## Files Modified

- `drk-api/src/lib.rs` - Added command schema types and ExecuteCommand event
- `drk-core/src/manager.rs` - Added get_all_plugin_commands() method
- `drk-cli/src/main.rs` - Implemented dynamic clap building and command routing
- `plugins/drk-basic/src/lib.rs` - Implemented get_commands() and command execution
- `plugins/drk-logger/src/lib.rs` - Added logging for ExecuteCommand event

## Future Enhancements

Potential improvements for the command schema system:

1. **Subcommands**: Support nested command structures (e.g., `drk git commit`)
2. **Argument Validation**: Add regex patterns or custom validators to `CommandArg`
3. **Default Values**: Allow specifying default values for optional arguments
4. **Aliases**: Support command and argument aliases
5. **Conflicts**: Define mutually exclusive arguments
6. **Environment Variables**: Map arguments to environment variables
7. **Completion**: Generate shell completion scripts from schemas

## Testing

The implementation has been tested with:
- ✅ Required arguments (echo --message)
- ✅ Optional arguments (greet --name)
- ✅ Default values (greet without --name)
- ✅ Help text generation
- ✅ Multiple plugins with multiple commands
- ✅ Event system integration

## Conclusion

The Command Schema system successfully solves the FFI boundary problem while providing a clean, type-safe interface for plugins to declare commands. The implementation is production-ready and serves as the foundation for the plugin-based CLI architecture.