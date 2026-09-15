//! Renderers for subsystems, their lifecycle ports and the variant connectors.
//!
//! Part of the renderer set: one module per Simulink library group.

#![cfg(feature = "egui")]

use super::continuous::{LEVEL_PULSE, reset_spec};
use crate::connection_targets::active_variant_port_index;
use crate::model::Block;
use crate::simulink_libraries::types::RenderContext;
use eframe::egui::{Painter, Rect};

/// Vertical fraction (from the top) at which the reinit port sits on a
/// `ShowSubsystemReinitializePorts` subsystem.  Must match the constant in
/// `egui_app::ui::signal_routing`.
const REINIT_PORT_FRAC: f32 = 0.12;

/// Vertical fraction (from the top) at which the separator line is drawn on a
/// `ShowSubsystemReinitializePorts` subsystem; data inputs are distributed in
/// the region below it.  Must match the constant in
/// `egui_app::ui::signal_routing`.
const REINIT_SEP_FRAC: f32 = 0.25;

/// Marks the input port whose "label" is the reset pictogram: line art drawn
/// by the block's renderer rather than a text label.  [`crate::simulink_libraries::render::port_label`]
/// suppresses it so the marker never reaches the screen as text.
pub const RESET_PORT: &str = "\u{1}reset";

/// Marks the input port drawn with the enable pictogram (a square pulse).
pub const ENABLE_PORT: &str = "\u{1}enable";

/// The rising-edge trigger pictogram: a step with the arrow head halfway up
/// its vertical edge.  Also used for the trigger port of a triggered subsystem.
pub const RISING_EDGE: &str =
    "p 0.05,0.90 0.45,0.90 0.45,0.15 0.90,0.15; p 0.32,0.72 0.45,0.48 0.58,0.72";

/// The falling-edge counterpart of [`RISING_EDGE`].
pub const FALLING_EDGE: &str =
    "p 0.05,0.15 0.45,0.15 0.45,0.90 0.90,0.90; p 0.32,0.33 0.45,0.57 0.58,0.33";

/// A pulse whose rising and falling edges both carry an arrow head.
pub const EITHER_EDGE: &str = concat!(
    "p 0.05,0.90 0.30,0.90 0.30,0.15 0.70,0.15 0.70,0.90 0.95,0.90;",
    "p 0.19,0.70 0.30,0.48 0.41,0.70; p 0.59,0.35 0.70,0.57 0.81,0.35"
);

/// Static renderer for a SubSystem.
///
/// A plain subsystem has **no** icon in Simulink (what looks like one is a
/// preview of its contents), so this only paints the symbols the contents
/// impose: the enable/trigger pictograms beneath the control ports on the top
/// edge, the for-each stack, and the lifecycle pictogram plus event name of a
/// contained `EventListener`.  It always reports "handled" so the generic icon
/// path never stamps a placeholder on a subsystem.
pub fn static_subsystem(
    painter: &Painter,
    block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let content = SubsystemContent::of(block);
    let mut spec = String::new();

    // The control pictograms sit under the top-edge ports they belong to, and
    // reuse the port pictograms of the Integrator/Delay reset ports.
    let controls = control_port_glyphs(&content);
    let size = (rect.width() / (controls.len() as f32 + 1.0))
        .min(rect.height() * 0.34)
        .min(16.0 * ctx.font_scale)
        .max(4.0);
    for (index, control) in controls.iter().enumerate() {
        let x = rect.left() + (index as f32 + 1.0) / (controls.len() as f32 + 1.0) * rect.width();
        let glyph = Rect::from_min_size(
            eframe::egui::pos2(x - size * 0.5, rect.top() + size * 0.2),
            eframe::egui::vec2(size, size),
        );
        crate::egui_app::render::draw_plot_icon(
            painter,
            &glyph,
            ctx.font_scale,
            control,
            ctx.text_color,
            None,
        );
    }
    // Lifecycle event ports enter on the input side, above the data inputs,
    // with their pictogram and event name beside them.
    let events = subsystem_event_input_glyphs(block);
    let _event_count = subsystem_event_input_count(block);
    let mirrored = block.block_mirror.unwrap_or(false);
    let side = crate::egui_app::geometry::port_side_for("in", mirrored);
    let reinit = is_reinit_subsystem(block);

    if reinit {
        // ShowSubsystemReinitializePorts: the reinit port sits in its own
        // section at the top of the input side, a horizontal separator line
        // spans the full block width beneath it, and the data inputs are
        // distributed in the lower section.
        let sep_y = rect.top() + REINIT_SEP_FRAC * rect.height();
        let stroke = eframe::egui::Stroke::new((1.4 * ctx.font_scale).max(0.75), ctx.border_color);
        painter.line_segment(
            [
                eframe::egui::pos2(rect.left(), sep_y),
                eframe::egui::pos2(rect.right(), sep_y),
            ],
            stroke,
        );
        let size = (rect.height() * REINIT_SEP_FRAC * 0.6)
            .min(rect.width() * 0.34)
            .min(14.0 * ctx.font_scale)
            .max(4.0);
        // Use the resolved event glyphs when available; fall back to the
        // generic reinit pictogram + "reinit" label when the subsystem
        // contents are not loaded.
        let fallback = format!(
            "{}; t 2.20,0.50,0.50 reinit",
            event_port_glyph(&EventKind::Reinitialize)
        );
        let glyphs: Vec<&str> = if !events.is_empty() {
            events.iter().map(String::as_str).collect()
        } else {
            vec![fallback.as_str()]
        };
        for event in &glyphs {
            let y = rect.top() + REINIT_PORT_FRAC * rect.height();
            let x = if mirrored {
                rect.right() - size * 1.2
            } else {
                rect.left() + size * 0.2
            };
            let glyph = Rect::from_min_size(
                eframe::egui::pos2(x, y - size * 0.5),
                eframe::egui::vec2(size, size),
            );
            crate::egui_app::render::draw_plot_icon(
                painter,
                &glyph,
                ctx.font_scale,
                event,
                ctx.text_color,
                None,
            );
        }
    } else if !events.is_empty() {
        let data_ins = block
            .port_counts
            .as_ref()
            .and_then(|counts| counts.ins)
            .unwrap_or(0);
        let total_ins = data_ins + events.len() as u32;
        let size = (rect.height() / (total_ins as f32 + 1.0))
            .min(rect.width() * 0.34)
            .min(14.0 * ctx.font_scale)
            .max(4.0);
        for (index, event) in events.iter().enumerate() {
            let y = crate::egui_app::geometry::port_anchor_pos(
                *rect,
                side,
                index as u32 + 1,
                Some(total_ins),
            )
            .y;
            let x = if mirrored {
                rect.right() - size * 1.2
            } else {
                rect.left() + size * 0.2
            };
            let glyph = Rect::from_min_size(
                eframe::egui::pos2(x, y - size * 0.5),
                eframe::egui::vec2(size, size),
            );
            crate::egui_app::render::draw_plot_icon(
                painter,
                &glyph,
                ctx.font_scale,
                event,
                ctx.text_color,
                None,
            );
        }
    }
    if content.for_each {
        // Stacked copies of the same block – one per element of the input.
        spec.push_str(concat!(
            "t 0.62,0.22,0.22 N;",
            "r 0.52,0.26 0.74,0.66; r 0.44,0.34 0.66,0.74; r 0.36,0.42 0.58,0.82"
        ));
    }
    if let Some(event) = content.event.as_ref() {
        // Simulink heads a function subsystem with the lifecycle pictogram of
        // the event its EventListener responds to, and the event's name.
        spec.push_str(match event.kind {
            // Circular arrow.
            EventKind::Reset => "sa 0.18,0.34,0.13,0.80,1.70;",
            // Bar fully inside the ring.  The ring is a closed arc so it keeps
            // the radius of the other lifecycle pictograms in a wide block.
            EventKind::Terminate => "s 0.18,0.34,0.13,0.00,1.00; p 0.18,0.26 0.18,0.42;",
            // Power symbol: the bar breaks through the gap at the top of the ring.
            EventKind::Initialize => "s 0.18,0.34,0.13,0.80,1.70; p 0.18,0.16 0.18,0.34;",
            // Both at once: the power symbol drawn with the reset arrow head.
            EventKind::Reinitialize => "sa 0.18,0.34,0.13,0.80,1.70; p 0.18,0.16 0.18,0.34;",
        });
        spec.push_str(&format!("t 0.62,0.34,0.30 {};", event.caption));
    }

    if !spec.is_empty() {
        crate::egui_app::render::draw_plot_icon(
            painter,
            rect,
            ctx.font_scale,
            &spec,
            ctx.text_color,
            None,
        );
    }
    true
}

/// Static renderer for a standalone `EnablePort`: the same square pulse the
/// containing subsystem shows above its enable port.
pub fn static_enable_port(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    crate::egui_app::render::draw_plot_icon(
        painter,
        rect,
        ctx.font_scale,
        LEVEL_PULSE,
        ctx.text_color,
        ctx.port_label_widths,
    );
    true
}

/// Static renderer for a standalone `TriggerPort`: the edge pictogram of its
/// `TriggerType`, matching the one on the containing subsystem's trigger port.
pub fn static_trigger_port(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let spec =
        reset_spec(ctx.metadata.get("TriggerType").or(Some("rising"))).unwrap_or(RISING_EDGE);
    crate::egui_app::render::draw_plot_icon(
        painter,
        rect,
        ctx.font_scale,
        spec,
        ctx.text_color,
        ctx.port_label_widths,
    );
    true
}

/// Static renderer for an `EventListener`: the lifecycle pictogram of the event
/// it responds to – the same one the subsystem containing it is headed with –
/// over the event's name.
pub fn static_event_listener(
    painter: &Painter,
    block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let event = SubsystemEvent::of(block);
    let glyph_height = rect.height() * 0.65;
    let side = glyph_height.min(rect.width());
    let glyph = Rect::from_center_size(
        eframe::egui::pos2(rect.center().x, rect.top() + glyph_height * 0.5),
        eframe::egui::vec2(side, glyph_height),
    );
    crate::egui_app::render::draw_plot_icon(
        painter,
        &glyph,
        ctx.font_scale,
        event_port_glyph(&event.kind),
        ctx.text_color,
        None,
    );
    let caption = Rect::from_min_max(
        eframe::egui::pos2(rect.left(), rect.top() + glyph_height),
        rect.max,
    );
    crate::egui_app::render::draw_plot_icon(
        painter,
        &caption,
        ctx.font_scale,
        &format!("t 0.50,0.50,0.80 {}", event.caption),
        ctx.text_color,
        ctx.port_label_widths,
    );
    true
}

/// Static renderer for the ResetPort block: the pictogram of the edge it
/// resets on – the same one its subsystem shows at the reset port it adds –
/// followed by the `R` annotation the subsystem draws beside it.  The block is
/// small, so the pictogram is drawn in a narrower sub-rect so the `R` at spec
/// x = 1.32 lands inside the block to its right, matching the subsystem reset
/// port's relative layout.  The divisor 1.6 accounts for the 10% margin
/// `compute_icon_available_rect` subtracts from each side, keeping the `R`
/// comfortably inside the block while shrinking the pictogram slightly.
pub fn static_reset_port(
    painter: &Painter,
    _block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let pictogram =
        reset_spec(ctx.metadata.get("ResetTriggerType").or(Some("rising"))).unwrap_or(RISING_EDGE);
    let spec = format!("{pictogram}; t 1.32,0.78,0.50 R");
    // Map spec x = 1.32 onto ~82% of the block width by using a sub-rect
    // whose width is `rect.width() / 1.6`.  After the 10% margin
    // `compute_icon_available_rect` subtracts, the `R` center lands at ~91%
    // of the block width — safely inside.  Center it vertically so the
    // pictogram keeps its full height.
    let sub_w = rect.width() / 1.6;
    // Shift the sub-rect left so the `R` (at spec x = 1.32, which maps to the
    // right end of the sub-rect) sits comfortably inside the block rather
    // than on its right border.
    let sub_rect = Rect::from_min_size(
        eframe::egui::pos2(
            rect.center().x - sub_w * 0.5 - rect.width() * 0.06,
            rect.top(),
        ),
        eframe::egui::vec2(sub_w, rect.height()),
    );
    crate::egui_app::render::draw_plot_icon(
        painter,
        &sub_rect,
        ctx.font_scale,
        &spec,
        ctx.text_color,
        ctx.port_label_widths,
    );
    true
}

/// The pictograms of the subsystem's top-edge control ports, in the order
/// Simulink places them: enable, trigger, reset, lifecycle event.  A reset port
/// is annotated with `R` and an event port with the event's name, both drawn
/// beside the pictogram.
fn control_port_glyphs(content: &SubsystemContent) -> Vec<String> {
    let mut glyphs: Vec<String> = Vec::new();
    if content.enabled {
        glyphs.push(LEVEL_PULSE.to_string());
    }
    if let Some(trigger) = content.triggered {
        glyphs.push(trigger.to_string());
    }
    if let Some(reset) = content.reset {
        glyphs.push(format!("{reset}; t 1.32,0.78,0.50 R"));
    }
    glyphs
}

/// The pictograms of the lifecycle event ports a subsystem carries on its
/// *input* side, top to bottom, each followed by the event's name – how
/// Simulink draws the reinitialize/reset port of a subsystem that contains such
/// a function.
pub fn subsystem_event_input_glyphs(block: &Block) -> Vec<String> {
    let content = SubsystemContent::of(block);
    content
        .event_port
        .iter()
        .map(|event| {
            format!(
                "{}; t 2.20,0.50,0.50 {}",
                event_port_glyph(&event.kind),
                event.caption
            )
        })
        .collect()
}

/// How many lifecycle event ports enter the subsystem on its input side, above
/// the data inputs.  Falls back to `<PortCounts event=…/>` when the contents
/// are not loaded or the nested EventListener is not found.
pub fn subsystem_event_input_count(block: &Block) -> u32 {
    let from_counts = || {
        block
            .port_counts
            .as_ref()
            .and_then(|counts| counts.event)
            .unwrap_or(0)
    };
    if block.subsystem.is_none() {
        return from_counts();
    }
    let from_content = u32::from(SubsystemContent::of(block).event_port.is_some());
    if from_content > 0 {
        from_content
    } else {
        // The subsystem is loaded but the nested EventListener was not found
        // (e.g. its own subsystem ref was not resolved).  Fall back to the
        // PortCounts `event` attribute so the port is still counted.
        from_counts()
    }
}

/// Whether the block carries `ShowSubsystemReinitializePorts = on`, meaning
/// Simulink draws the reinit port in its own section at the top of the input
/// side, a horizontal separator line beneath it, and the data inputs below.
pub fn is_reinit_subsystem(block: &Block) -> bool {
    block
        .properties
        .get("ShowSubsystemReinitializePorts")
        .is_some_and(|v| v.trim().eq_ignore_ascii_case("on"))
}

/// The lifecycle pictogram of an event port, drawn inside its own square.
fn event_port_glyph(kind: &EventKind) -> &'static str {
    match kind {
        EventKind::Reset => "sa 0.50,0.58,0.34,0.80,1.70;",
        EventKind::Terminate => "s 0.50,0.58,0.34,0.00,1.00; p 0.50,0.36 0.50,0.80;",
        EventKind::Initialize => "s 0.50,0.58,0.34,0.80,1.70; p 0.50,0.14 0.50,0.58;",
        EventKind::Reinitialize => "sa 0.50,0.58,0.34,0.80,1.70; p 0.50,0.14 0.50,0.58;",
    }
}

/// The endpoint types of the subsystem's top-edge ports, left to right, in the
/// same order [`control_port_glyphs`] draws their pictograms.  This is what
/// turns an `enable:1` / `trigger:1` endpoint – both numbered 1, each in its
/// own type's numbering – into the slot it occupies on the edge.  Falls back to
/// the model's `<PortCounts>` for a subsystem whose contents are not loaded.
pub fn subsystem_control_port_types(block: &Block) -> Vec<&'static str> {
    if block.subsystem.is_none() {
        let Some(counts) = block.port_counts.as_ref() else {
            return Vec::new();
        };
        return [
            ("enable", counts.enable),
            ("trigger", counts.trigger),
            ("reset", counts.reset),
        ]
        .into_iter()
        .flat_map(|(port_type, count)| std::iter::repeat_n(port_type, count.unwrap_or(0) as usize))
        .collect();
    }

    let content = SubsystemContent::of(block);
    let mut types = Vec::new();
    if content.enabled {
        types.push("enable");
    }
    if content.triggered.is_some() {
        types.push("trigger");
    }
    if content.reset.is_some() {
        types.push("reset");
    }
    types
}

/// How many ports a subsystem carries on its top edge, derived from the blocks
/// it contains so the port markers and their pictograms always agree.
pub fn subsystem_control_port_count(block: &Block) -> u32 {
    subsystem_control_port_types(block).len() as u32
}

/// The parts of a subsystem's contents that shape how Simulink draws it.
struct SubsystemContent {
    enabled: bool,
    /// The pictogram of a contained `TriggerPort`, per its `TriggerType`.
    triggered: Option<&'static str>,
    /// The pictogram of a contained `ResetPort`, per its `ResetTriggerType`.
    reset: Option<&'static str>,
    /// The lifecycle event a nested function subsystem exposes on the parent's
    /// top edge (`<PortCounts event="1"/>`).
    event_port: Option<SubsystemEvent>,
    for_each: bool,
    /// The lifecycle event a contained `EventListener` responds to – what
    /// distinguishes initialize/reset/reinitialize/terminate function
    /// subsystems from one another.
    event: Option<SubsystemEvent>,
}

struct SubsystemEvent {
    kind: EventKind,
    caption: String,
}

#[derive(PartialEq, Eq)]
enum EventKind {
    Initialize,
    Reinitialize,
    Reset,
    Terminate,
}

impl SubsystemEvent {
    /// Simulink captions Initialize and Terminate functions with the event
    /// itself; Reset and Reinitialize functions carry a user-chosen event name.
    fn of(listener: &Block) -> SubsystemEvent {
        let event_type = listener
            .properties
            .get("EventType")
            .map(|s| s.trim())
            .unwrap_or("Initialize");
        let name = listener
            .properties
            .get("EventName")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());
        match event_type.to_ascii_lowercase().as_str() {
            "reset" => SubsystemEvent {
                kind: EventKind::Reset,
                caption: name.unwrap_or("reset").to_string(),
            },
            "reinitialize" => SubsystemEvent {
                kind: EventKind::Reinitialize,
                caption: name.unwrap_or("reinit").to_string(),
            },
            "terminate" => SubsystemEvent {
                kind: EventKind::Terminate,
                caption: "terminate".to_string(),
            },
            other => SubsystemEvent {
                kind: EventKind::Initialize,
                caption: other.to_string(),
            },
        }
    }
}

impl SubsystemContent {
    fn of(block: &Block) -> Self {
        let mut content = SubsystemContent {
            enabled: false,
            triggered: None,
            reset: None,
            event_port: None,
            for_each: false,
            event: None,
        };
        if let Some(system) = block.subsystem.as_deref() {
            for child in &system.blocks {
                // A function subsystem nested inside this one surfaces its
                // event as a port on this block's top edge.
                if let Some(nested) = child.subsystem.as_deref()
                    && let Some(listener) = nested
                        .blocks
                        .iter()
                        .find(|inner| inner.block_type == "EventListener")
                {
                    content.event_port = Some(SubsystemEvent::of(listener));
                }
                match child.block_type.as_str() {
                    "EnablePort" => content.enabled = true,
                    "ResetPort" => {
                        content.reset = Some(
                            reset_spec(
                                child
                                    .properties
                                    .get("ResetTriggerType")
                                    .map(String::as_str)
                                    .or(Some("rising")),
                            )
                            .unwrap_or(RISING_EDGE),
                        )
                    }
                    "TriggerPort" => {
                        content.triggered = Some(
                            reset_spec(
                                child
                                    .properties
                                    .get("TriggerType")
                                    .map(String::as_str)
                                    .or(Some("rising")),
                            )
                            .unwrap_or(RISING_EDGE),
                        )
                    }
                    "ForEach" => content.for_each = true,
                    "EventListener" => content.event = Some(SubsystemEvent::of(child)),
                    _ => {}
                }
            }
        }
        content
    }
}

/// Draw a small square at each port position.  Returns `true` (the renderer
/// fully handles the interior).
pub fn static_variant_connector(
    painter: &Painter,
    block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    draw_variant_port_squares(painter, block, rect, ctx, None);
    true
}

/// Live renderer: fills the active port square and draws a connecting lever
/// line from the active input to the active output.
pub fn live_variant_connector(
    _app: &mut crate::egui_app::state::SubsystemApp,
    ui: &mut eframe::egui::Ui,
    block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
) -> bool {
    let active = active_variant_port_index(block);
    if active.is_none() {
        // Can't determine the active variant — fall back to static.
        return false;
    }
    draw_variant_port_squares(
        &ui.painter().with_clip_rect(*rect),
        block,
        rect,
        ctx,
        active,
    );
    draw_variant_lever(
        &ui.painter().with_clip_rect(*rect),
        block,
        rect,
        ctx,
        active,
    );
    true
}

/// Draw the small port squares for a variant routing block.
/// When `active_port` is `Some`, that port's square is filled dark.
fn draw_variant_port_squares(
    painter: &Painter,
    block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
    active_port: Option<u32>,
) {
    let stroke_w = (1.5 * ctx.font_scale).clamp(1.0, 3.0);
    let square_size = (6.0 * ctx.font_scale).clamp(3.0, 10.0);
    let half = square_size / 2.0;
    let col_active = eframe::egui::Color32::from_rgb(32, 32, 32);
    let col_inactive = eframe::egui::Color32::from_rgb(110, 110, 110);
    let fill_active = eframe::egui::Color32::from_rgb(32, 32, 32);

    let (max_in, max_out) = variant_port_counts(block);
    let coords = ctx.port_y;

    // Draw input port squares (left side).
    for idx in 1..=max_in {
        let y = coords
            .and_then(|c| c.inputs.get(&idx).copied())
            .unwrap_or_else(|| {
                crate::egui_app::geometry::port_anchor_pos(
                    *rect,
                    crate::egui_app::geometry::PortSide::In,
                    idx,
                    Some(max_in),
                )
                .y
            });
        let center = eframe::egui::pos2(rect.left() + half + 1.0, y);
        let is_active = active_port.is_some_and(|p| p == idx);
        let color = if is_active { col_active } else { col_inactive };
        if is_active {
            painter.rect_filled(
                eframe::egui::Rect::from_center_size(
                    center,
                    eframe::egui::vec2(square_size, square_size),
                ),
                0.0,
                fill_active,
            );
        } else {
            painter.rect_stroke(
                eframe::egui::Rect::from_center_size(
                    center,
                    eframe::egui::vec2(square_size, square_size),
                ),
                0.0,
                eframe::egui::Stroke::new(stroke_w, color),
                eframe::egui::StrokeKind::Inside,
            );
        }
    }

    // Draw output port squares (right side).
    for idx in 1..=max_out {
        let y = coords
            .and_then(|c| c.outputs.get(&idx).copied())
            .unwrap_or_else(|| {
                crate::egui_app::geometry::port_anchor_pos(
                    *rect,
                    crate::egui_app::geometry::PortSide::Out,
                    idx,
                    Some(max_out),
                )
                .y
            });
        let center = eframe::egui::pos2(rect.right() - half - 1.0, y);
        let is_active = active_port.is_some_and(|p| p == idx);
        let color = if is_active { col_active } else { col_inactive };
        if is_active {
            painter.rect_filled(
                eframe::egui::Rect::from_center_size(
                    center,
                    eframe::egui::vec2(square_size, square_size),
                ),
                0.0,
                fill_active,
            );
        } else {
            painter.rect_stroke(
                eframe::egui::Rect::from_center_size(
                    center,
                    eframe::egui::vec2(square_size, square_size),
                ),
                0.0,
                eframe::egui::Stroke::new(stroke_w, color),
                eframe::egui::StrokeKind::Inside,
            );
        }
    }
}

/// Draw the connecting lever line from the active input port to the active
/// output port.
fn draw_variant_lever(
    painter: &Painter,
    block: &Block,
    rect: &Rect,
    ctx: &RenderContext<'_>,
    active_port: Option<u32>,
) {
    let Some(active) = active_port else {
        return;
    };
    let stroke_w = (1.5 * ctx.font_scale).clamp(1.0, 3.0);
    let col = eframe::egui::Color32::from_rgb(32, 32, 32);
    let square_size = (6.0 * ctx.font_scale).clamp(3.0, 10.0);
    let half = square_size / 2.0;
    let (max_in, max_out) = variant_port_counts(block);
    let coords = ctx.port_y;

    let (start, end) = if max_in == 1 && max_out > 1 {
        // VariantStart / VariantSink: 1 input → N outputs.
        // Active port index refers to the output port.
        let in_y = coords
            .and_then(|c| c.inputs.get(&1).copied())
            .unwrap_or_else(|| {
                crate::egui_app::geometry::port_anchor_pos(
                    *rect,
                    crate::egui_app::geometry::PortSide::In,
                    1,
                    Some(max_in),
                )
                .y
            });
        let out_y = coords
            .and_then(|c| c.outputs.get(&active).copied())
            .unwrap_or_else(|| {
                crate::egui_app::geometry::port_anchor_pos(
                    *rect,
                    crate::egui_app::geometry::PortSide::Out,
                    active,
                    Some(max_out),
                )
                .y
            });
        (
            eframe::egui::pos2(rect.left() + half + 1.0 + half, in_y),
            eframe::egui::pos2(rect.right() - half - 1.0 - half, out_y),
        )
    } else if max_in > 1 && max_out == 1 {
        // VariantEnd / VariantSource: N inputs → 1 output.
        // Active port index refers to the input port.
        let in_y = coords
            .and_then(|c| c.inputs.get(&active).copied())
            .unwrap_or_else(|| {
                crate::egui_app::geometry::port_anchor_pos(
                    *rect,
                    crate::egui_app::geometry::PortSide::In,
                    active,
                    Some(max_in),
                )
                .y
            });
        let out_y = coords
            .and_then(|c| c.outputs.get(&1).copied())
            .unwrap_or_else(|| {
                crate::egui_app::geometry::port_anchor_pos(
                    *rect,
                    crate::egui_app::geometry::PortSide::Out,
                    1,
                    Some(max_out),
                )
                .y
            });
        (
            eframe::egui::pos2(rect.left() + half + 1.0 + half, in_y),
            eframe::egui::pos2(rect.right() - half - 1.0 - half, out_y),
        )
    } else {
        return;
    };

    painter.line_segment([start, end], eframe::egui::Stroke::new(stroke_w, col));
}

/// Get the (input_count, output_count) for a variant routing block.
fn variant_port_counts(block: &Block) -> (u32, u32) {
    let ins = block
        .port_counts
        .as_ref()
        .and_then(|c| c.ins)
        .unwrap_or_else(|| block.ports.iter().filter(|p| p.port_type == "in").count() as u32);
    let outs = block
        .port_counts
        .as_ref()
        .and_then(|c| c.outs)
        .unwrap_or_else(|| block.ports.iter().filter(|p| p.port_type == "out").count() as u32);
    (ins, outs)
}
