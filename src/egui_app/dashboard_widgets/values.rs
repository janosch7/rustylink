//! Reading the widget configuration and the live value out of a block.

#![cfg(feature = "egui")]

use super::style::{
    ACCENT, ACCENT_DARK, clamp_u8, parse_block_background_color, parse_color_property,
};
use crate::model::Block;
use eframe::egui::Color32;

// ─── Helpers ────────────────────────────────────────────────────────────

/// Read a block property as a string, falling back to a default.
pub(super) fn prop<'a>(block: &'a Block, name: &str, default: &'a str) -> &'a str {
    block
        .properties
        .get(name)
        .map(|s| s.as_str())
        .unwrap_or(default)
}

pub(super) fn checkbox_label(block: &Block) -> String {
    block
        .properties
        .get("Label")
        .or_else(|| block.properties.get("Text"))
        .cloned()
        .unwrap_or_else(|| "Label".to_string())
}

fn parse_range_property(block: &Block, keys: &[&str]) -> Option<f64> {
    keys.iter().find_map(|key| {
        block
            .properties
            .get(*key)
            .and_then(|value| value.trim().parse::<f64>().ok())
    })
}

pub(super) fn normalized_live_value(block: &Block, value: f64) -> f32 {
    let min =
        parse_range_property(block, &["Minimum", "ScaleMin", "LowerLimit", "Min"]).unwrap_or(0.0);
    let max =
        parse_range_property(block, &["Maximum", "ScaleMax", "UpperLimit", "Max"]).unwrap_or(100.0);
    if max <= min {
        return 0.0;
    }
    ((value - min) / (max - min)).clamp(0.0, 1.0) as f32
}

pub(super) fn discrete_live_index(value: f64, option_count: usize) -> usize {
    if option_count == 0 {
        return 0;
    }
    value
        .round()
        .clamp(0.0, (option_count.saturating_sub(1)) as f64) as usize
}

pub(super) fn combo_box_label(block: &Block, index: usize) -> String {
    option_labels(block)
        .get(index)
        .cloned()
        .unwrap_or_else(|| format!("Label {}", index + 1))
}

pub(super) fn option_label_value_pairs(block: &Block) -> Vec<(String, Option<f64>)> {
    block
        .properties
        .get("Values")
        .map(|values| {
            values
                .split(['\n', ',', ';'])
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .map(|entry| {
                    if let Some((label, value)) = entry.rsplit_once(':') {
                        let parsed = value.trim().parse::<f64>().ok();
                        (label.trim().to_string(), parsed)
                    } else {
                        (entry.to_string(), None)
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub(super) fn checkbox_state_from_value(block: &Block, live_value: f64) -> bool {
    let pairs = option_label_value_pairs(block);
    if pairs.len() >= 2 {
        let off = pairs[0].1.unwrap_or(0.0);
        let on = pairs[1].1.unwrap_or(1.0);
        return (live_value - on).abs() <= (live_value - off).abs();
    }
    live_value >= 0.5
}

pub(super) fn gauge_range(block: &Block) -> (f64, f64) {
    let min =
        parse_range_property(block, &["Minimum", "ScaleMin", "LowerLimit", "Min"]).unwrap_or(0.0);
    let max =
        parse_range_property(block, &["Maximum", "ScaleMax", "UpperLimit", "Max"]).unwrap_or(100.0);
    if min < max { (min, max) } else { (0.0, 100.0) }
}

pub(super) fn format_scale_value(value: f64) -> String {
    if (value.fract()).abs() < 1e-9 {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
    }
}

pub(super) fn option_labels(block: &Block) -> Vec<String> {
    let labels = option_label_value_pairs(block)
        .into_iter()
        .map(|(label, _)| label)
        .collect::<Vec<_>>();
    if labels.is_empty() {
        vec![
            "Label 1".to_string(),
            "Label 2".to_string(),
            "Label 3".to_string(),
        ]
    } else {
        labels
    }
}

#[allow(dead_code)]
pub(super) fn discrete_option_items(block: &Block) -> Vec<(String, f64)> {
    let pairs = option_label_value_pairs(block);
    if pairs.is_empty() {
        return option_labels(block)
            .into_iter()
            .enumerate()
            .map(|(index, label)| (label, index as f64))
            .collect();
    }
    pairs
        .into_iter()
        .enumerate()
        .map(|(index, (label, value))| (label, value.unwrap_or(index as f64)))
        .collect()
}

#[allow(dead_code)]
pub(super) fn discrete_selected_index(block: &Block, live_value: f64) -> usize {
    let options = discrete_option_items(block);
    if options.is_empty() {
        return 0;
    }
    options
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| {
            (left.1 - live_value)
                .abs()
                .partial_cmp(&(right.1 - live_value).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(index, _)| index)
        .unwrap_or(0)
}

pub(super) fn push_button_visuals(block: &Block, live_value: Option<f64>) -> (Color32, f64) {
    let on_color = parse_color_property(block, &["IconOnColor"]).unwrap_or(ACCENT);
    let off_color = parse_color_property(block, &["IconOffColor"]).unwrap_or(ACCENT_DARK);
    let off_value = block
        .properties
        .get("OffValue")
        .and_then(|v| v.trim().parse::<f64>().ok())
        .unwrap_or(0.0);
    let color = match live_value {
        Some(value) if (value - off_value).abs() > f64::EPSILON => on_color,
        Some(_) => off_color,
        None => off_color,
    };
    (color, off_value)
}

pub(super) fn configured_dashboard_value(block: &Block) -> Option<f64> {
    block
        .current_setting
        .as_ref()
        .and_then(|value| value.trim().parse::<f64>().ok())
        .or_else(|| {
            block
                .value
                .as_ref()
                .and_then(|value| value.trim().parse::<f64>().ok())
        })
        .or_else(|| {
            block
                .properties
                .get("Value")
                .and_then(|value| value.trim().parse::<f64>().ok())
        })
}

fn parse_bracketed_numbers(raw: &str) -> Vec<f64> {
    let trimmed = raw.trim();
    let values = if let Some(idx) = trimmed.find('[') {
        &trimmed[idx..]
    } else {
        trimmed
    };
    values
        .replace(['[', ']'], " ")
        .replace(';', ",")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<f64>().ok())
        .collect()
}

fn lamp_default_color(block: &Block) -> Color32 {
    parse_color_property(block, &["DefaultColor"])
        .or_else(|| parse_block_background_color(block))
        .unwrap_or(Color32::from_rgb(192, 192, 192))
}

fn lamp_state_colors(block: &Block) -> Option<Vec<(f64, Color32)>> {
    let raw = block.properties.get("States")?;
    let mut cells = raw.split('|');
    let values = parse_bracketed_numbers(cells.next()?);
    let colors = parse_bracketed_numbers(cells.next()?);
    if values.is_empty() || colors.len() < 3 {
        return None;
    }
    let color_values = colors
        .chunks(3)
        .filter(|chunk| chunk.len() == 3)
        .map(|chunk| {
            Color32::from_rgb(
                clamp_u8(chunk[0] as f32),
                clamp_u8(chunk[1] as f32),
                clamp_u8(chunk[2] as f32),
            )
        })
        .collect::<Vec<_>>();
    if color_values.is_empty() {
        return None;
    }
    Some(values.into_iter().zip(color_values).collect())
}

pub(super) fn lamp_color_for_value(block: &Block, value: Option<f64>) -> Color32 {
    let default = lamp_default_color(block);
    let Some(value) = value else {
        return default;
    };
    lamp_state_colors(block)
        .and_then(|states| {
            states.into_iter().find_map(|(state_value, color)| {
                if (value - state_value).abs() <= 1e-9 {
                    Some(color)
                } else {
                    None
                }
            })
        })
        .unwrap_or(default)
}

#[cfg(feature = "dashboard")]
pub(super) fn dashboard_input_control_kind(block: &Block) -> Option<&'static str> {
    if !matches!(
        block.dashboard_binding,
        Some(crate::model::DashboardBinding::ParamSource { .. })
    ) {
        return None;
    }

    // The control kind is data on the block's definition, not a block-type match.
    crate::simulink_libraries::resolve_definition(block)
        .dashboard_control
        .map(|kind| kind.as_str())
}

#[cfg(feature = "dashboard")]
pub(super) fn dashboard_control_storage_key(block: &Block) -> String {
    block.sid.clone().unwrap_or_else(|| block.name.clone())
}

pub(super) fn format_dashboard_scalar_with_options(
    value: f64,
    display_options: Option<&crate::live_values::LiveValueDisplayOptions>,
) -> String {
    let mut entry = crate::live_values::LiveValueEntry::new(crate::live_values::LiveValue::new(
        vec![1],
        crate::live_values::LiveValueList::Float64(vec![value]),
    ));
    if let Some(options) = display_options {
        entry = entry.with_display(options.clone());
    }
    entry.formatted_text()
}
