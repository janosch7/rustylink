//! Dashboard / HMI blocks.
//!
//! These blocks demonstrate the live/static renderer split: when live mode is
//! OFF the [`renderers::static_dashboard`] adapter draws the widget icon; when
//! live mode is ON [`renderers::live_dashboard`] draws the live value overlay
//! (gauges, switches, displays reflecting the current value).

#![cfg(feature = "egui")]

use crate::simulink_libraries::renderers;
use crate::simulink_libraries::types::{
    IOPorts, SimulinkBlockDefinition, SimulinkIcon, SimulinkShape,
};

const fn icon(glyph: &'static str) -> SimulinkIcon {
    SimulinkIcon::Utf8(glyph)
}

/// A dashboard widget definition with both static and live renderers wired up.
const fn widget(
    block_type: &'static str,
    glyph: &'static str,
    inputs: IOPorts,
) -> SimulinkBlockDefinition {
    SimulinkBlockDefinition::new(block_type, "Dashboard")
        .with_ports(inputs, IOPorts::None)
        .with_icon(icon(glyph))
        .with_static_renderer(renderers::static_dashboard)
        .with_live_renderer(renderers::live_dashboard)
}

pub static BLOCKS: &[SimulinkBlockDefinition] = &[
    // Value displays.
    SimulinkBlockDefinition::new("Display", "Dashboard")
        .with_description("Display the value of the connected signal")
        .with_ports(IOPorts::Fixed(1), IOPorts::None)
        .with_icon(icon("📟"))
        .with_live_renderer(renderers::live_dashboard),
    widget("DisplayBlock", "📟", IOPorts::Fixed(1)),
    // Controls (inputs).
    widget("PushButtonBlock", "⏻", IOPorts::None),
    widget("Checkbox", "☑", IOPorts::None),
    widget("ComboBox", "▾", IOPorts::None),
    widget("EditField", "✎", IOPorts::None),
    widget("RadioButtonGroup", "◉", IOPorts::None),
    widget("SliderBlock", "⎯●", IOPorts::None),
    widget("SliderSwitchBlock", "⇅", IOPorts::None),
    widget("ToggleSwitchBlock", "⏼", IOPorts::None),
    widget("RockerSwitchBlock", "⏻", IOPorts::None),
    widget("RotarySwitchBlock", "◎", IOPorts::None),
    widget("KnobBlock", "◎", IOPorts::None),
    // Gauges / indicators (outputs/displays).
    widget("CircularGaugeBlock", "◔", IOPorts::Fixed(1)),
    widget("SemiCircularGaugeBlock", "◑", IOPorts::Fixed(1)),
    widget("QuarterGaugeBlock", "◕", IOPorts::Fixed(1)),
    widget("LinearGaugeBlock", "▮", IOPorts::Fixed(1)),
    widget("LampBlock", "💡", IOPorts::Fixed(1)),
    // DashboardScope's live view is an interactive liveplot tile owned by the
    // UI; the static fallback is a simple waveform glyph.
    SimulinkBlockDefinition::new("DashboardScope", "Dashboard")
        .with_description("Plot connected signals on a dashboard scope")
        .with_ports(IOPorts::Fixed(1), IOPorts::None)
        .with_shape(SimulinkShape::Rectangle)
        .with_icon(icon("〰"))
        .with_static_renderer(renderers::static_scope),
];
