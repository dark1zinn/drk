// We export the manager so the CLI can use it
pub mod manager;

// Re-export commonly used items from the API for convenience, 
// though strict usage should usually depend on drk-api directly.
pub use drk_api::*;