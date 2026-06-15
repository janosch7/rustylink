#![cfg(feature = "egui")]

//! Custom font installation for the egui viewer/editor.

use eframe::egui;

/// Register the egui-phosphor icon font so [`SimulinkIcon::Phosphor`] block
/// icons render as real glyphs instead of missing-glyph boxes.
///
/// Phosphor glyphs are appended as a fallback to the proportional family, so
/// regular text is unaffected while the catalog's phosphor icon codepoints
/// resolve correctly.
///
/// [`SimulinkIcon::Phosphor`]: crate::simulink_libraries::types::SimulinkIcon::Phosphor
pub fn install(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    ctx.set_fonts(fonts);
}
