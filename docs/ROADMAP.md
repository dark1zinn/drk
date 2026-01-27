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

## 6. UI & UX Consistency Layer (New Roadmap Entries)

### [ ] Feature: Standardized Terminal Styling (`console`)

**Implementation:**

* **Where:** Re-export from `drk-api`.
* **Logic:** Use the `console` crate to provide a global "Style" guide. This allows us to handle `NO_COLOR` environment variables in one place.
* **API Exposure:** 
```rust
// In drk-api/src/lib.rs
pub use console;
pub fn style_primary(text: &str) -> console::StyledObject<&str> {
console::style(text).cyan().bold()
}
```


* **Benefit:** If we ever want to change the "drk theme," we change it in `drk-api`, and every plugin updates its look automatically upon its next compile.

### [ ] Feature: Unified Progress Reporting (`indicatif`)

**Implementation:**

* **Where:** `drk-api` defines a `ProgressContext` passed through `SystemEvent`.
* **Logic:** Instead of plugins creating their own progress bars (which can mess up the terminal if multiple plugins do it), plugins should request a bar from the `drk-core` via the `EventBus`.
* **Flow:**
1. Plugin sends `SystemEvent::RequestProgressBar`.
2. `drk-core` initializes an `indicatif::MultiProgress` and returns a handle.


* **Benefit:** Prevents "flickering" or overlapping progress bars when multiple background tasks occur.

### [ ] Feature: Diagnostic Error Reporting (`miette`)

**Implementation:**

* **Where:** Integrated into the `Result` type of the `Plugin` trait.
* **Logic:** Change the `Plugin` trait methods to return `miette::Result<T>`.
* **Formatting:**
```rust
#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("Plugin Config Error")]
pub struct PluginError {
    #[help]
    pub advice: String,
    // miette will render this with beautiful arrows and colors
}

```


* **Benefit:** When a plugin fails, the user gets a "compiler-grade" error report with specific advice, rather than a generic "Plugin failed" message.

---

## 7. Updated Progress Tracking

* [x] Define Workspace and Crate structure.
* [x] Implement FFI Plugin loading logic (`libloading`).
* [x] Create Type-safe Event Bus (Enum-based).
* [x] Implement Plugin Toggling and Metadata.
* [x] Update `drk-basic` to Dynamic Library format.
* [x] Implement `CommandSchema` for dynamic CLI argument parsing.
* [ ] **Next:** Re-export `console` in `drk-api` for UI consistency.
* [ ] **Next:** Integrate `indicatif` into `PluginManager` for managed progress bars.
* [ ] **Next:** Migrate `anyhow` to `miette` for advanced diagnostics.

---

### Implementation Note: `console` vs `colored`

I've chosen `console` for this roadmap because it is specifically designed to handle **terminal detection** (knowing if it's a pipe, a file, or a real terminal) better than `colored`. This is vital for a CLI tool that might be used in CI/CD pipelines where you don't want ANSI escape codes cluttering up log files.

**Would you like to start the code implementation for the `CommandSchema` now, or should we first set up the UI/Error crates in `drk-api`?**
