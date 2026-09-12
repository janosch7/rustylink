//! Resolution of the Simulink paths a block or line is connected to.
//!
//! Split by concern: the resolver drives the traversal, `routing` knows how the
//! signal-routing blocks forward a signal, `metadata` carries names and test
//! points along, and `graph` holds the pure model lookups.

mod debug;
mod graph;
mod metadata;
mod resolver;
mod routing;
mod target;
mod topology;
mod variants;

pub use debug::*;
pub use resolver::*;
pub use target::*;
pub use topology::*;
pub use variants::*;
