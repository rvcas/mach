//! SeaORM entities for Mach.
//!
//! Keep module paths stable so `db.get_schema_registry("machich::entity::*")`
//! can discover everything automatically.

pub mod config;
pub mod project;
pub mod todo;
pub mod workspace;

/// Convenience exports for downstream modules.
pub mod prelude {
    pub use super::config;
    pub use super::project;
    pub use super::todo;
    pub use super::workspace;
}
