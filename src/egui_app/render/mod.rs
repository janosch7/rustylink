//! Drawing a Simulink block on the egui canvas.
//!
//! The body shape, the icon and the labels of a block are drawn by `body`,
//! `icons` and `labels`; the remaining modules hold the blocks whose icon is
//! tied to their port geometry and therefore cannot be described by an icon
//! spec alone.

#![cfg(feature = "egui")]

mod block_type;
mod body;
mod goto_from;
mod icons;
mod labels;
mod logic_gates;
mod math_blocks;
mod switches;

pub use block_type::*;
pub use body::*;
pub use goto_from::*;
pub use icons::*;
pub use labels::*;
pub use logic_gates::*;
pub use math_blocks::*;
pub use switches::*;
