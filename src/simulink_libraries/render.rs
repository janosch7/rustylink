//! The single, general block-interior renderer.
//!
//! Both the viewer and the editor call [`render_block_interior`] to draw the
//! inside of a block (icon, symbol, gauge, live value, …).  There is no
//! block-type-specific code here: the behaviour is entirely driven by the
//! block's resolved [`SimulinkBlockDefinition`] and its optional static / live
//! renderer functions.

#![cfg(feature = "egui")]

use eframe::egui::{Color32, FontId, Painter, Rect};

use crate::model::Block;

use super::metadata::extract_metadata;
use super::resolver::resolve_definition;
use super::types::{BlockLabelPolicy, RenderContext, SimulinkBlockDefinition, SimulinkShape};

/// Parameters supplied by the UI for one interior render call.
///
/// These are the pieces of state the renderer cannot derive from the block
/// alone (live values/text, zoom, label widths, port geometry).
pub struct InteriorParams<'a> {
    pub live_mode: bool,
    pub font_scale: f32,
    pub name_font_factor: f32,
    pub live_value: Option<f64>,
    pub live_text: Option<&'a str>,
    pub live_display_options: Option<&'a crate::live_values::LiveValueDisplayOptions>,
    pub port_y: Option<&'a crate::egui_app::render::ComputedPortYCoordinates>,
    pub port_label_widths: Option<crate::egui_app::render::PortLabelMaxWidths>,
    /// Foreground/contrast color used for plain-text labels.
    pub text_color: Color32,
}

/// Render the interior of a block, driven entirely by its definition.
///
/// This is the painter-only path used by the editor and, in the viewer, for
/// every block that is **not** currently handled by its live renderer.  The
/// live renderer (which needs `&mut SubsystemApp`/`&mut Ui`) is dispatched by
/// the viewer before falling back here, so there is no live branch in this
/// function.
///
/// Dispatch order:
/// 1. `FilledBlack` shapes have no interior.
/// 2. The definition's `static_renderer` (if any).
/// 3. A metadata/fixed [`BlockLabelPolicy`] or `compute_instance_label`.
/// 4. The definition's icon (falling back to the generic icon path).
pub fn render_block_interior(
    painter: &Painter,
    block: &Block,
    rect: &Rect,
    params: &InteriorParams<'_>,
) {
    let def = resolve_definition(block);
    let metadata = extract_metadata(block, def);
    let ctx = RenderContext {
        live_mode: params.live_mode,
        font_scale: params.font_scale,
        name_font_factor: params.name_font_factor,
        metadata: &metadata,
        live_value: params.live_value,
        live_text: params.live_text,
        live_display_options: params.live_display_options,
        port_y: params.port_y,
        port_label_widths: params.port_label_widths,
        text_color: params.text_color,
    };

    // 1. Solid-fill blocks (BusCreator/BusSelector) draw nothing inside.
    if def.shape == SimulinkShape::FilledBlack {
        return;
    }

    // 2. Static renderer.
    if let Some(f) = def.static_renderer
        && f(painter, block, rect, &ctx)
    {
        return;
    }

    // 3. Textual block label.
    if let Some(label) = block_label_text(block, def, &metadata)
        && !label.is_empty()
    {
        let font = FontId::proportional(12.0 * ctx.font_scale);
        let galley = painter.layout_no_wrap(label, font, params.text_color);
        let pos = rect.center() - galley.size() * 0.5;
        painter.galley(pos, galley, params.text_color);
        return;
    }

    // 4. Icon.  The catalog definition is the source of truth: if it carries an
    // icon, draw it directly (this also wins over any stale config-map entry for
    // blocks that also exist in a bridged virtual library).
    if let Some(icon) = def.icon {
        let spec = super::config::icon_to_spec(icon);
        // Use the contrast color derived from the block's actual fill (which may
        // be the neutral gray of "less colorful" mode) so glyphs stay legible.
        crate::egui_app::render::draw_icon_spec(
            painter,
            rect,
            ctx.font_scale,
            &spec,
            params.text_color,
            ctx.port_label_widths,
        );
        return;
    }

    // No icon in the definition: a non-rectangular body (e.g. the Gain triangle)
    // already conveys the block's identity, so leave it empty rather than stamp a
    // noisy `?` placeholder.
    if def.shape != SimulinkShape::Rectangle {
        return;
    }

    // Rectangular & iconless: fall back to the legacy config-map icon path, which
    // is the single place that rasterises every icon kind and emits the `?`
    // fallback (plus a one-time warning) for unknown rectangular blocks.
    crate::egui_app::render::render_block_icon(
        painter,
        block,
        rect,
        ctx.font_scale,
        params.text_color,
        ctx.port_label_widths,
    );
}

/// Resolve the textual block label per the definition's policy.
fn block_label_text(
    block: &Block,
    def: &SimulinkBlockDefinition,
    metadata: &super::metadata::BlockMetadata,
) -> Option<String> {
    match def.block_label {
        BlockLabelPolicy::None => {}
        BlockLabelPolicy::Fixed(s) => return Some(s.to_string()),
        BlockLabelPolicy::MetadataDependent(f) => {
            if let Some(s) = f(block, metadata) {
                return Some(s);
            }
        }
    }
    def.compute_instance_label.and_then(|f| f(block))
}
