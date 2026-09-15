//! Icon renderers for the Sinks blocks and the data-store / state accessors.
//!
//! Part of the renderer set: one module per Simulink library group.

#![cfg(feature = "egui")]

use super::common::body_colors;
use crate::model::Block;
use crate::simulink_libraries::types::RenderContext;
use eframe::egui::{Painter, Rect};

/// Static renderer for the Data Store Read/Write blocks: the name of the store
/// they access, framed by the rules Simulink draws above and below it.
pub fn static_data_store_access(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let name = ctx
        .metadata
        .get("DataStoreName")
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or("A");
    let spec = format!("p 0.14,0.18 0.86,0.18; p 0.14,0.82 0.86,0.82; t 0.50,0.50,0.46 {name}");
    crate::egui_app::render::draw_plot_icon(
        painter,
        rect,
        ctx.font_scale,
        &spec,
        ctx.text_color,
        ctx.port_label_widths,
    );
    true
}

/// Static renderer for the blocks that write into another block's state or
/// parameters: a diamond carrying `x` (state) or `p` (parameter).  The block
/// they act on is named in the label Simulink prints beside the diamond.
pub fn static_state_parameter_access(
    painter: &Painter,
    block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let glyph = if block.block_type == "ParameterWriter" {
        "p"
    } else {
        "x"
    };
    let colors = body_colors(ctx);
    let center = rect.center();
    painter.add(eframe::egui::Shape::convex_polygon(
        vec![
            eframe::egui::pos2(center.x, rect.top()),
            eframe::egui::pos2(rect.right(), center.y),
            eframe::egui::pos2(center.x, rect.bottom()),
            eframe::egui::pos2(rect.left(), center.y),
        ],
        colors.fill,
        eframe::egui::Stroke::new((1.4 * ctx.font_scale).max(0.75), colors.border),
    ));
    painter.text(
        center,
        eframe::egui::Align2::CENTER_CENTER,
        glyph,
        eframe::egui::FontId::proportional(
            (rect.height() * 0.5)
                .min(24.0 * ctx.font_scale)
                .clamp(1.0, 24.0),
        ),
        colors.text,
    );
    if let Some(owner) = state_parameter_owner(ctx) {
        painter.text(
            eframe::egui::pos2(rect.right() + rect.width() * 0.25, center.y),
            eframe::egui::Align2::LEFT_CENTER,
            owner,
            eframe::egui::FontId::proportional(
                (rect.height() * 0.42).min(14.0 * ctx.font_scale).max(1.0),
            ),
            ctx.text_color,
        );
    }
    true
}

/// The block a State/Parameter Reader or Writer acts on, as Simulink prints it
/// beside the diamond: the owner block's own name – the trailing component of
/// the `../Delay` style path – and, for a parameter, the parameter it writes,
/// as in `Add Constant.Bias`.
fn state_parameter_owner(ctx: &RenderContext<'_>) -> Option<String> {
    owner_caption(
        ctx.metadata
            .get("StateOwnerBlock")
            .or_else(|| ctx.metadata.get("ParameterOwnerBlock")),
        ctx.metadata.get("ParameterName"),
    )
}

pub(super) fn owner_caption(owner_path: Option<&str>, parameter: Option<&str>) -> Option<String> {
    let path = owner_path.map(str::trim).filter(|path| !path.is_empty())?;
    let owner = path.rsplit('/').next().unwrap_or(path).trim();
    match parameter.map(str::trim) {
        Some(parameter) if !parameter.is_empty() => Some(format!("{owner}.{parameter}")),
        _ => Some(owner.to_string()),
    }
}

/// Static renderer for Scope / DashboardScope blocks (waveform glyph).
pub fn static_scope(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    _ctx: &RenderContext<'_>,
) -> bool {
    crate::egui_app::ui::update::paint_scope_glyph(painter, rect);
    true
}
