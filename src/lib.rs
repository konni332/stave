//! Compile-time typestate validation for type-safe builder patterns in Rust.
//!
//! `stave` provides an architectural solution to uninitialized configuration
//! bugs by leveraging the Rust type system to shift verification errors from
//! runtime to compile time. Instead of relying on runtime checks or panicking
//! when a required parameter is omitted, `stave` tracks the initialization
//! state of a struct at the type level, preventing dependent methods from
//! compiling until the structural requirements are fully satisfied.
//!
//! # Core Mechanics
//!
//! The framework operates via two complementary procedural attribute macros:
//!
//! * [`builder`]: Applied to a struct definition. It analyzes field metadata,
//!   wraps optional fields in an `Option<T>`, and synthesizes internal marker types
//!   (e.g., `__FieldUnset` and `__FieldSet`) for required fields. It adds corresponding
//!   generic parameters to the struct to securely track whether each required field
//!   contains a value.
//! * [`methods`]: Applied to an `impl` block for that struct. It synthesizes
//!   boilerplate public setters (`set_{field_name}`) for unannotated fields, generates
//!   type-restricted getters, and processes state transformations via two sub-attributes:
//!   `#[sets(...)]` and `#[requires(...)].`
//!
//! # Macro Evaluation Ordering
//!
//! Because procedural macro attribute invocations do not naturally share state or
//! token streams during compilation, `stave` bridges this boundary using an in-process
//! compile-time registry. The `#[builder]` macro evaluates the layout schema and records
//! the structural configuration to an internal global cache. The `#[methods]` macro
//! subsequently queries this metadata registry by struct identifier to safely synthesize
//! state-transition code.
//!
//! Due to this architecture, **the `#[builder]` attribute must always appear in the
//! source stream prior to its corresponding `#[methods]` block**. Standard source layouts
//! where the struct definition precedes its implementation block satisfy this rule.
//! Split implementations across multiple `#[methods]` blocks or distinct modules are
//! currently unsupported.
//!
//! # Quick Start Example
//!
//! ```rust
//! use stave::{builder, methods};
//!
//! #[builder]
//! struct Configurator {
//!     #[stave(required)]
//!     identity: String,
//!     #[stave(required)]
//!     target_port: u16,
//!     timeout_ms: u64, // Defaults to optional, wrapped in Option<u64>
//! }
//!
//! #[methods]
//! impl Configurator {
//!     // Custom setter specifying flexible input transformations
//!     #[sets(identity)]
//!     fn set_identity(self, val: impl Into<String>) -> String {
//!         val.into()
//!     }
//!
//!     // Arbitrary user-defined method restricted by compile-time typestates
//!     #[requires(identity, target_port)]
//!     fn establish_connection(self) {
//!         // Inside here, self.identity() and self.target_port() safely yield references
//!         println!("Connecting to {} on port {}", self.identity(), self.target_port());
//!     }
//! }
//!
//! fn main() {
//!     // Compiles flawlessly:
//!     Configurator::new()
//!         .set_identity("node_alpha")
//!         .set_target_port(9000) // Boilerplate generated automatically
//!         .establish_connection();
//!
//!     // Will NOT compile (triggers E0599: method `establish_connection` not found):
//!     // Configurator::new().set_identity("node_beta").establish_connection();
//! }
//! ```

pub use stave_macros::{builder, methods};
