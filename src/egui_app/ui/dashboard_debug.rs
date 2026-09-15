//! Console dumps describing how a dashboard block binds to a signal.

#[cfg(feature = "dashboard")]
use super::live_values::block_live_text;
#[cfg(feature = "dashboard")]
use crate::egui_app::state::SubsystemApp;

// Print connected block/signal information for a dashboard UI block.
///
/// Dashboard blocks do not use traditional signal lines; they use
/// `BindingPersistence` references to `.mxarray` files that describe
/// which block parameter they write to or which signal they read.
///
/// The resolved binding is stored in `Block::dashboard_binding` during
/// archive loading.
///
/// For blocks that use traditional signal lines instead of BindingPersistence
/// (e.g., `Display`, `Scope`), this function falls back to scanning the
/// current subsystem's lines for connections to/from this block.
pub(super) fn print_dashboard_connected_signals(
    block: &crate::model::Block,
    lines: &[crate::model::Line],
) {
    println!(
        "  [Dashboard UI] Block '{}' (type: {})",
        block.name, block.block_type
    );

    match &block.dashboard_binding {
        Some(crate::model::DashboardBinding::ParamSource {
            block_path,
            param_name,
            uuid,
            ..
        }) => {
            println!(
                "    → writes param '{}' on block '{}' (uuid: {})",
                param_name, block_path, uuid
            );
        }
        Some(crate::model::DashboardBinding::SignalSpec {
            block_path,
            signal_name,
            uuid,
            ..
        }) => {
            println!(
                "    ← reads signal '{}' from block '{}' (uuid: {})",
                signal_name, block_path, uuid
            );
        }
        None => {
            if crate::simulink_libraries::traits::reads_signal_line(&block.block_type) {
                println!(
                    "    · line-based block (no BindingPersistence expected) for SID {}",
                    block.sid.as_deref().unwrap_or("<none>")
                );
            } else {
                println!(
                    "    · dashboard binding missing or unresolved for SID {}",
                    block.sid.as_deref().unwrap_or("<none>")
                );
                print_dashboard_binding_debug(block);
            }
            // Fall back to line-based connection scanning
            print_line_based_connections(block, &[], lines);
        }
    }
}

pub(super) fn print_dashboard_binding_debug(block: &crate::model::Block) {
    let binding_persistence = block.properties.get("BindingPersistence");
    let has_binding_ref =
        binding_persistence.is_some() || block.ref_properties.contains("BindingPersistence");
    println!(
        "    · tag={} library_source={:?} library_block_path={:?}",
        block.tag_name, block.library_source, block.library_block_path
    );
    println!(
        "    · BindingPersistence present: {}{}",
        if has_binding_ref { "yes" } else { "no" },
        binding_persistence
            .map(|value| format!(" ({value})"))
            .unwrap_or_default()
    );

    let property_debug = collect_dashboard_debug_properties(&block.properties);
    if property_debug.is_empty() {
        println!("    · no binding-related block properties found");
    } else {
        println!(
            "    · binding-related block properties: {}",
            property_debug.join(", ")
        );
    }

    let instance_debug = block
        .instance_data
        .as_ref()
        .map(|instance_data| collect_dashboard_debug_properties(&instance_data.properties))
        .unwrap_or_default();
    if instance_debug.is_empty() {
        println!("    · no binding-related instance-data properties found");
    } else {
        println!(
            "    · binding-related instance-data properties: {}",
            instance_debug.join(", ")
        );
    }

    let ref_debug: Vec<&str> = block
        .ref_properties
        .iter()
        .filter(|name| is_dashboard_debug_property(name))
        .map(|name| name.as_str())
        .collect();
    if ref_debug.is_empty() {
        println!("    · no binding-related ref-backed properties found");
    } else {
        println!(
            "    · binding-related ref-backed properties: {}",
            ref_debug.join(", ")
        );
    }
}

pub(super) fn collect_dashboard_debug_properties(
    properties: &indexmap::IndexMap<String, String>,
) -> Vec<String> {
    properties
        .iter()
        .filter(|(name, _)| is_dashboard_debug_property(name))
        .take(10)
        .map(|(name, value)| format!("{name}={value}"))
        .collect()
}

pub(super) fn is_dashboard_debug_property(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("binding")
        || lower.contains("uuid")
        || lower.contains("signal")
        || lower.contains("param")
        || lower.contains("dashboard")
        || lower.contains("widget")
        || lower.contains("hmi")
        || lower == "sid"
        || lower.contains("element")
        || lower.contains("ref")
}

#[cfg(feature = "dashboard")]
pub(super) fn dashboard_live_value(app: &SubsystemApp, block: &crate::model::Block) -> Option<f64> {
    let binding = block.dashboard_binding.as_ref()?;
    let entry = app.live_values.get(binding.uuid())?;
    let selector_index = match binding {
        crate::model::DashboardBinding::ParamSource { target_path, .. } => {
            target_path.element_index_zero_based()
        }
        crate::model::DashboardBinding::SignalSpec { .. } => None,
    };

    selector_index
        .filter(|_| entry.value.data.len() > 1)
        .and_then(|index| entry.f64_at(index))
        .or_else(|| entry.first_f64())
}

#[cfg(feature = "dashboard")]
pub(super) fn dashboard_input_control_kind(block: &crate::model::Block) -> Option<&'static str> {
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
pub(super) fn dashboard_widget_value(app: &SubsystemApp, block: &crate::model::Block) -> f64 {
    let dashboard_value = dashboard_live_value(app, block);
    let block_value = app
        .live_block_values
        .get(&app.live_value_key_for_block(block))
        .and_then(crate::live_values::LiveValueEntry::first_f64);

    dashboard_value
        .or(block_value)
        .or_else(|| block_live_text(app, block).and_then(|value| value.trim().parse::<f64>().ok()))
        .or_else(|| {
            block
                .current_setting
                .as_ref()
                .and_then(|value| value.trim().parse::<f64>().ok())
        })
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
        .unwrap_or_else(|| match dashboard_input_control_kind(block) {
            Some("scalar") => dashboard_scalar_range(block).0,
            _ => 0.0,
        })
}

#[cfg(feature = "dashboard")]
pub(super) fn dashboard_scalar_range(block: &crate::model::Block) -> (f64, f64) {
    fn parse_property(block: &crate::model::Block, keys: &[&str]) -> Option<f64> {
        keys.iter().find_map(|key| {
            block
                .properties
                .get(*key)
                .and_then(|value| value.trim().parse::<f64>().ok())
        })
    }

    let min = parse_property(block, &["Minimum", "ScaleMin", "LowerLimit", "Min"]);
    let max = parse_property(block, &["Maximum", "ScaleMax", "UpperLimit", "Max"]);
    let min = min.unwrap_or(0.0);
    let max = max.unwrap_or(100.0);
    if min < max { (min, max) } else { (0.0, 100.0) }
}

/// Scan lines in the current subsystem for connections to/from the given block.
pub(super) fn print_line_based_connections(
    block: &crate::model::Block,
    blocks: &[crate::model::Block],
    lines: &[crate::model::Line],
) {
    let block_sid = match &block.sid {
        Some(s) => s.as_str(),
        None => {
            println!("    (no SID — cannot scan line connections)");
            return;
        }
    };

    // Helper: collect all destination SIDs from a line (including branches).
    fn collect_dst_sids(line: &crate::model::Line) -> Vec<&str> {
        let mut sids = Vec::new();
        if let Some(ref dst) = line.dst {
            sids.push(dst.sid.as_str());
        }
        fn branch_dsts<'a>(branches: &'a [crate::model::Branch], acc: &mut Vec<&'a str>) {
            for b in branches {
                if let Some(ref dst) = b.dst {
                    acc.push(dst.sid.as_str());
                }
                branch_dsts(&b.branches, acc);
            }
        }
        branch_dsts(&line.branches, &mut sids);
        sids
    }

    // Build a SID→name lookup for blocks in this subsystem.
    let block_name_by_sid: std::collections::HashMap<&str, &str> = blocks
        .iter()
        .filter_map(|b| b.sid.as_deref().map(|s| (s, b.name.as_str())))
        .collect();

    let mut found_any = false;

    for line in lines {
        let signal_name = line.name.as_deref().unwrap_or("<unnamed>");

        // Check if this block is a source of the line
        if let Some(ref src) = line.src
            && src.sid == block_sid
        {
            let dst_sids = collect_dst_sids(line);
            for dsid in &dst_sids {
                let dst_name = block_name_by_sid.get(dsid).copied().unwrap_or("?");
                println!(
                    "    → drives signal '{}' to block '{}' (dst SID {})",
                    signal_name, dst_name, dsid
                );
                found_any = true;
            }
        }

        // Check if this block is a destination of the line
        let all_dsts = collect_dst_sids(line);
        if all_dsts.contains(&block_sid)
            && let Some(ref src) = line.src
        {
            let src_name = block_name_by_sid
                .get(src.sid.as_str())
                .copied()
                .unwrap_or("?");
            println!(
                "    ← receives signal '{}' from block '{}' (src SID {})",
                signal_name, src_name, src.sid
            );
            found_any = true;
        }
    }

    if !found_any {
        println!("    (no signal-line connections found in current subsystem)");
    }
}
