//! Drawing and interaction for the Simulink Dashboard blocks.
//!
//! `statics` draws a widget as it appears in a static diagram, `live` binds it
//! to a value and makes it interactive; both share the look defined in `style`
//! and the configuration reading in `values`.

#![cfg(feature = "egui")]

mod controls;
mod live;
mod painters;
mod statics;
mod style;
mod values;

pub use controls::*;
pub use live::*;
pub use statics::*;
