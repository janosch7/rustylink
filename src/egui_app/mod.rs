//! Egui-based interactive viewer for Simulink systems (feature = "egui").
//!
//! This module splits the original monolithic implementation into smaller
//! submodules to improve readability and maintainability.

#![cfg(feature = "egui")]

use eframe::egui::{self, Align2, Rect};

pub mod dashboard_widgets;
mod geometry;
pub mod icon_assets;
mod navigation;
mod render;
pub mod scope_widget;
mod state;
pub mod text;
mod ui;

// Re-export geometry items needed by the editor module
pub use geometry::{
    PortSide, endpoint_pos_maybe_mirrored, parse_block_rect, parse_rect_str, port_anchor_pos,
    port_indicator_positions,
};
pub use navigation::{
    collect_subsystems_paths, resolve_subsystem_by_path, resolve_subsystem_by_vec,
};
pub use render::wrap_text_to_max_width;

// Helpers which are useful for integration tests
pub use render::{PortLabelMaxWidths, compute_icon_available_rect};
// Interior renderer registry access (needed by dashboard visualization tests)
pub use render::{InteriorRendererFn, get_interior_renderer};
#[cfg(feature = "dashboard")]
pub use state::ScopePopout;
pub use state::{
    BlockContextMenuItem, BlockDialog, BlockDialogButton, ChartView, SignalContextMenuItem,
    SignalDialog, SignalDialogButton, SubsystemApp, SubsystemEntities,
};
#[cfg(feature = "dashboard")]
pub use state::{DashboardControlEvent, DashboardControlValue};
pub use text::{highlight_query_job, matlab_syntax_job};
pub use ui::zoom_controls::show_zoom_controls;
pub use ui::{
    ClickAction, UpdateResponse, apply_update_response, show_info_windows, update, update_with_info,
};
// Expose the canonical color utility module for reuse by the editor.
pub use ui::colors;

// Expose a couple of internal helpers for use by integration tests.
pub use ui::helpers::{block_dialog_title, clean_display_string};
// SVG parsing helper (also needed by some tests)
pub use render::embedded_egui_sans_fontdb;

pub(crate) fn port_label_display_name(
    block: &crate::model::Block,
    index: u32,
    is_input: bool,
    cfg: &crate::block_types::BlockTypeConfig,
) -> String {
    let mirrored = block.block_mirror.unwrap_or(false);
    let logical_is_input = if mirrored { !is_input } else { is_input };

    let fallback_name = || {
        let names = if logical_is_input {
            &cfg.input_port_names
        } else {
            &cfg.output_port_names
        };
        if index > 0 && (index as usize) <= names.len() {
            names[(index - 1) as usize].clone()
        } else {
            format!("{}{}", if is_input { "In" } else { "Out" }, index)
        }
    };

    block
        .ports
        .iter()
        .filter(|port| {
            port.port_type == if logical_is_input { "in" } else { "out" }
                && port.index.unwrap_or(0) == index
        })
        .find_map(|port| {
            port.properties
                .get("Name")
                .cloned()
                .or_else(|| port.properties.get("name").cloned())
                .map(|name| name.trim().to_string())
                .filter(|name| !name.is_empty())
        })
        .unwrap_or_else(fallback_name)
}

pub fn get_block_type_cfg(block: &crate::model::Block) -> crate::block_types::BlockTypeConfig {
    viewer_block_type_override(block).unwrap_or_else(|| render::get_block_type_cfg(block))
}

pub fn render_block_icon(
    painter: &egui::Painter,
    block: &crate::model::Block,
    rect: &Rect,
    font_scale: f32,
    port_label_widths: Option<PortLabelMaxWidths>,
) {
    let Some(cfg) = viewer_block_type_override(block) else {
        render::render_block_icon(painter, block, rect, font_scale, port_label_widths);
        return;
    };

    let Some(icon) = cfg.icon else {
        return;
    };

    match icon {
        crate::block_types::IconSpec::Utf8(glyph) => {
            if glyph.is_empty() {
                return;
            }
            let avail = compute_icon_available_rect(rect, font_scale, port_label_widths);
            if avail.width() <= 1.0 || avail.height() <= 1.0 {
                return;
            }
            let color = ui::colors::contrast_color(ui::colors::block_base_color(block, &cfg));
            let font_px = avail.width().min(avail.height()).max(1.0);
            painter.text(
                avail.center(),
                Align2::CENTER_CENTER,
                glyph,
                egui::FontId::proportional(font_px),
                color,
            );
        }
        crate::block_types::IconSpec::Svg(_) => {
            render::render_block_icon(painter, block, rect, font_scale, port_label_widths);
        }
    }
}

fn viewer_block_type_override(
    block: &crate::model::Block,
) -> Option<crate::block_types::BlockTypeConfig> {
    match block.block_type.as_str() {
        "Display" => Some(crate::block_types::BlockTypeConfig {
            icon: Some(crate::block_types::IconSpec::Utf8("📟")),
            show_input_port_labels: false,
            show_output_port_labels: false,
            known: true,
            default_ins: 1,
            ..Default::default()
        }),
        "DisplayBlock" => Some(crate::block_types::BlockTypeConfig {
            icon: Some(crate::block_types::IconSpec::Utf8("📟")),
            show_input_port_labels: false,
            show_output_port_labels: false,
            known: true,
            ..Default::default()
        }),
        "Mux" => Some(crate::block_types::BlockTypeConfig {
            icon: Some(crate::block_types::IconSpec::Utf8("☰")),
            show_input_port_labels: false,
            show_output_port_labels: false,
            known: true,
            default_ins: 2,
            default_outs: 1,
            ..Default::default()
        }),
        "Demux" => Some(crate::block_types::BlockTypeConfig {
            icon: Some(crate::block_types::IconSpec::Utf8("☰")),
            show_input_port_labels: false,
            show_output_port_labels: false,
            known: true,
            default_ins: 1,
            default_outs: 2,
            ..Default::default()
        }),
        "BusCreator" => Some(crate::block_types::BlockTypeConfig {
            icon: Some(crate::block_types::IconSpec::Utf8("☰")),
            show_input_port_labels: false,
            show_output_port_labels: false,
            known: true,
            default_ins: 2,
            default_outs: 1,
            ..Default::default()
        }),
        "BusSelector" => Some(crate::block_types::BlockTypeConfig {
            icon: Some(crate::block_types::IconSpec::Utf8("☰")),
            show_input_port_labels: false,
            show_output_port_labels: false,
            known: true,
            default_ins: 1,
            default_outs: 2,
            ..Default::default()
        }),
        "ComplexToRealImag" => Some(crate::block_types::BlockTypeConfig {
            icon: Some(crate::block_types::IconSpec::Utf8("")),
            known: true,
            default_ins: 1,
            default_outs: 2,
            input_port_names: vec!["Re+Im".to_string()],
            output_port_names: vec!["Re".to_string(), "Im".to_string()],
            ..Default::default()
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::port_label_display_name;

    #[test]
    fn port_labels_do_not_fall_back_to_propagated_signals() {
        let mut block =
            crate::editor::operations::create_default_block("SubSystem", "SubSystem", 0, 0, 1, 1);
        block.ports = vec![crate::model::Port {
            port_type: "out".to_string(),
            index: Some(1),
            properties: indexmap::IndexMap::from_iter([(
                "PropagatedSignals".to_string(),
                "ConnectedSignal".to_string(),
            )]),
        }];

        let cfg = crate::egui_app::get_block_type_cfg(&block);
        assert_eq!(port_label_display_name(&block, 1, false, &cfg), "Out1");
    }

    #[test]
    fn port_labels_keep_explicit_port_names() {
        let mut block =
            crate::editor::operations::create_default_block("SubSystem", "SubSystem", 0, 0, 1, 1);
        block.ports = vec![crate::model::Port {
            port_type: "out".to_string(),
            index: Some(1),
            properties: indexmap::IndexMap::from_iter([(
                "Name".to_string(),
                "SubsystemOutput".to_string(),
            )]),
        }];

        let cfg = crate::egui_app::get_block_type_cfg(&block);
        assert_eq!(
            port_label_display_name(&block, 1, false, &cfg),
            "SubsystemOutput"
        );
    }
}
