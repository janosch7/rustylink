//! Live values, live text and the manual-switch toggle of the running model.

#[cfg(feature = "dashboard")]
use super::dashboard_debug::{dashboard_live_value, dashboard_widget_value};
use crate::egui_app::navigation::resolve_subsystem_by_vec;
use crate::egui_app::state::{SubsystemApp, resolve_subsystem_by_vec_mut};

#[cfg(feature = "dashboard")]
pub(super) fn block_live_text(app: &SubsystemApp, block: &crate::model::Block) -> Option<String> {
    app.live_block_values
        .get(&app.live_value_key_for_block(block))
        .map(crate::live_values::LiveValueEntry::formatted_text)
}

#[cfg(not(feature = "dashboard"))]
pub(super) fn block_live_text(_app: &SubsystemApp, _block: &crate::model::Block) -> Option<String> {
    None
}

pub(super) fn toggle_manual_switch_setting(
    app: &mut SubsystemApp,
    block: &crate::model::Block,
) -> Option<bool> {
    let block_sid = block.sid.as_ref()?;

    // Read the current switch state from the live parameter value (which, in
    // live mode, is what the display reflects).  Fall back to the model's
    // static `current_setting` when no live value has arrived yet.
    //
    // Done with an immutable borrow of `app` before the mutable
    // `resolve_subsystem_by_vec_mut` borrow below.
    let live_enabled = app
        .live_block_values
        .get(&app.live_value_key_for_block(block))
        .and_then(crate::live_values::LiveValueEntry::first_f64)
        .map(|value| value >= 0.5);
    let model_enabled = resolve_subsystem_by_vec(&app.root, &app.path)
        .and_then(|system| {
            system
                .blocks
                .iter()
                .find(|candidate| candidate.sid.as_deref() == Some(block_sid.as_str()))
        })
        .and_then(|block| block.current_setting.as_deref())
        .map(|setting| setting == "1");
    let current_enabled = live_enabled.or(model_enabled).unwrap_or(false);

    let path = app.path.clone();
    let system = resolve_subsystem_by_vec_mut(&mut app.root, &path)?;
    let live_block = system
        .blocks
        .iter_mut()
        .find(|candidate| candidate.sid.as_ref() == Some(block_sid))?;

    let enabled = !current_enabled;
    live_block.current_setting = Some(if enabled { "1" } else { "0" }.to_string());
    app.view_cache.invalidate();
    Some(enabled)
}

pub(super) fn uses_live_value_text(block_type: &str) -> bool {
    crate::simulink_libraries::traits::shows_value_text(block_type)
}

pub(super) fn should_render_live_text(live_mode_enabled: bool, block_type: &str) -> bool {
    live_mode_enabled && uses_live_value_text(block_type)
}

pub(super) fn should_use_mask_display(block: &crate::model::Block) -> bool {
    block.mask.is_some()
        && block
            .mask_display_text
            .as_ref()
            .is_some_and(|text| !text.trim().is_empty())
}

pub(crate) fn manual_switch_setting_from_live_value(value: f64) -> &'static str {
    if value >= 0.5 { "1" } else { "0" }
}

#[cfg(feature = "dashboard")]
pub(super) fn block_live_value(app: &SubsystemApp, block: &crate::model::Block) -> Option<f64> {
    app.live_block_values
        .get(&app.live_value_key_for_block(block))
        .and_then(crate::live_values::LiveValueEntry::first_f64)
        .or_else(|| dashboard_live_value(app, block))
}

pub(super) fn branch_hits_sid(branch: &crate::model::Branch, sid: &str) -> bool {
    if branch.dst.as_ref().is_some_and(|dst| dst.sid == sid) {
        return true;
    }
    branch
        .branches
        .iter()
        .any(|child| branch_hits_sid(child, sid))
}

#[cfg(feature = "dashboard")]
pub(super) fn first_input_signal_name(
    block: &crate::model::Block,
    lines: &[crate::model::Line],
) -> Option<String> {
    let sid = block.sid.as_deref()?;
    lines.iter().find_map(|line| {
        let direct = line.dst.as_ref().is_some_and(|dst| dst.sid == sid);
        let branched = line
            .branches
            .iter()
            .any(|branch| branch_hits_sid(branch, sid));
        if direct || branched {
            line.name.clone().filter(|name| !name.trim().is_empty())
        } else {
            None
        }
    })
}

#[cfg(feature = "dashboard")]
pub(super) fn scope_title_for_block(
    block: &crate::model::Block,
    lines: &[crate::model::Line],
) -> String {
    if let Some(crate::model::DashboardBinding::SignalSpec { signal_name, .. }) =
        block.dashboard_binding.as_ref()
        && !signal_name.trim().is_empty()
    {
        return signal_name.clone();
    }

    first_input_signal_name(block, lines).unwrap_or_else(|| block.name.clone())
}

#[cfg(feature = "dashboard")]
pub(super) fn update_scope_live_sample(
    app: &mut SubsystemApp,
    block: &crate::model::Block,
    lines: &[crate::model::Line],
) {
    if !crate::simulink_libraries::traits::shows_live_trace(&block.block_type) {
        return;
    }

    let Some(value) = block_live_value(app, block) else {
        return;
    };

    let scope_key = app.scope_key_for_block(block);
    let scope_title = scope_title_for_block(block, lines);
    let mut scopes = app.scope_instances.lock().unwrap();
    let scope = scopes.entry(scope_key.clone()).or_insert_with(|| {
        crate::egui_app::scope_widget::MiniScope::new((
            app.instance_id,
            "scope",
            scope_key.as_str(),
        ))
    });
    scope.set_signal_name(scope_title);
    scope.push_sample(value);
}

#[cfg(not(feature = "dashboard"))]
pub(super) fn block_live_value(_app: &SubsystemApp, _block: &crate::model::Block) -> Option<f64> {
    None
}

/// Live value fed to a block's unified `LiveRendererFn`.
///
/// Interactive dashboard controls (those carrying a `dashboard_control` kind)
/// resolve a value through the selector-aware [`dashboard_widget_value`] so the
/// widget reflects/edits the bound parameter; every other block uses the plain
/// signal value.
#[cfg(feature = "dashboard")]
pub(super) fn live_block_value_for_renderer(
    app: &SubsystemApp,
    block: &crate::model::Block,
    def: &crate::simulink_libraries::types::SimulinkBlockDefinition,
) -> Option<f64> {
    if def.dashboard_control.is_some() {
        Some(dashboard_widget_value(app, block))
    } else {
        block_live_value(app, block)
    }
}

#[cfg(not(feature = "dashboard"))]
pub(super) fn live_block_value_for_renderer(
    app: &SubsystemApp,
    block: &crate::model::Block,
    _def: &crate::simulink_libraries::types::SimulinkBlockDefinition,
) -> Option<f64> {
    block_live_value(app, block)
}
