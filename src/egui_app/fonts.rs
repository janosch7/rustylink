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

    // Bundle DejaVu Sans as a broad-coverage fallback so the technical/math
    // glyphs used by block icons (✓ ☰ ∠ ∿ ⊥ ⇒ superscripts …) render as real
    // glyphs instead of missing-glyph boxes. It is appended *after* the default
    // family and phosphor, so regular Latin text keeps its normal font and
    // DejaVu only supplies glyphs nothing earlier can provide.
    const DEJAVU_SANS: &[u8] = include_bytes!("../../assets/fonts/DejaVuSans.ttf");
    fonts.font_data.insert(
        "DejaVuSans".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(DEJAVU_SANS)),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .push("DejaVuSans".to_owned());
    }

    ctx.set_fonts(fonts);
}
