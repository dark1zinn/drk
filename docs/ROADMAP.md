# Project "drk" Development Manifest

## 1. Project Overview

`drk` is a modular, plugin-based CLI tool written in Rust. It utilizes a **Dynamic Runtime Architecture**, allowing features and commands to be added via compiled shared libraries (`.so`, `.dll`, `.dylib`) without recompiling the main binary.

### Key Goals

* **Decoupled Features:** The core CLI is just an orchestrator; logic lives in plugins.
* **Type-Safe Pub/Sub:** A centralized `SystemEvent` enum ensures plugins can communicate safely.
* **Lifecycle Management:** Plugins can be toggled on/off via configuration, except for "essential" plugins.
* **Unified Config:** A single TOML file managed by the CLI, namespaced by plugin names.

---

## 2. Workspace Architecture

The project is organized as a Cargo Workspace:

* **`drk-api`**: The "Contract." Contains the `Plugin` trait, the `SystemEvent` enum, and FFI macros. Both the CLI and Plugins depend on this.
* **`drk-core`**: The "Engine." Contains the `PluginManager`, handles dynamic loading via `libloading`, and manages the event loop.
* **`drk-cli`**: The "Shell." The entry-point binary that loads the config and triggers the Manager.
* **`plugins/`**: A directory for dynamic library crates (e.g., `drk-basic`, `drk-logger`).

---

## 3. Current Implementation State

### The Event System (`drk-api`)

We moved away from a generic `Any` approach to a type-safe enum to prevent FFI memory issues:

```rust
pub enum SystemEvent {
    Startup,
    PreCommand { name: String, args: Vec<String> },
    PostCommand { name: String, success: bool },
    Custom { 
        source: String, 
        event: String, 
        payload: Option<Arc<dyn Any + Send + Sync>> 
    },
}

```

### The Plugin Manager (`drk-core`)

The manager scans for `cdylib` files and instantiates them. It handles the "Double-Drop" safety requirement (dropping the plugin instance before unloading the library).

### Metadata & Toggles

Every plugin provides:

* `name`, `version`, `author`, `description`.
* `essential`: A boolean. If `false`, the CLI checks `config.toml` to see if `enabled = true` before loading.

---

## 4. Current Codebase (Core Logic)

### `Plugin` Trait

```rust
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    fn on_load(&mut self) -> Result<()>;
    fn handle_event(&mut self, event: &SystemEvent, ctx: &mut Context) -> Result<()>;
}

```

### `Context` Struct

Passed to plugins to allow controlled interaction with the host:

```rust
pub struct Context<'a> {
    pub config: &'a mut HashMap<String, toml::Value>,
    pub event_sender: &'a mut dyn FnMut(SystemEvent),
}

```

---

## 5. Next Step: Command Schema System

### The Problem

Currently, the CLI binary doesn't know what commands a plugin provides until after the plugin is loaded. Furthermore, we cannot pass `clap::Command` objects across the FFI boundary safely because `clap` does not have a stable ABI.

### The Solution

We will implement a **Command Schema**:

1. **Define a Serializble Schema:** Create a simple struct in `drk-api` (e.g., `PluginCommand`) that describes a command name, description, and arguments (as simple strings/enums).
2. **Trait Update:** Add `fn get_commands(&self) -> Vec<PluginCommand>` to the `Plugin` trait.
3. **Dynamic Clap Building:** In `drk-cli`, we will iterate through all loaded plugins, read their `PluginCommand` schemas, and use them to build the `clap::Command` tree at runtime.
4. **Routing:** When a user runs `drk greet --name "Gemini"`, the CLI will identify "greet" belongs to `drk-basic` and fire a `SystemEvent::ExecuteCommand` with the parsed matches.

---

### Progress Tracking

* [x] Define Workspace and Crate structure.
* [x] Implement FFI Plugin loading logic (`libloading`).
* [x] Create Type-safe Event Bus (Enum-based).
* [x] Implement Plugin Toggling and Metadata.
* [x] Update `drk-basic` to Dynamic Library format.
* [x] Implement `CommandSchema` for dynamic CLI argument parsing.
