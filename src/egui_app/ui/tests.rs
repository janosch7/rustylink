//! Behaviour tests for the viewer canvas: live values, line styling and dashboards.

use super::dashboard_debug::{
    dashboard_input_control_kind, dashboard_live_value, dashboard_widget_value,
};
use super::line_style::{
    line_has_testpoint, line_stroke_width, line_testpoint_marker_position, resolved_line_label,
};
use super::live_values::{
    manual_switch_setting_from_live_value, should_render_live_text, should_use_mask_display,
};
use crate::connection_targets::{
    ConnectionTarget, ConnectionTargetOrigin, ConnectionTargetResolve, ConnectionTargetResolver,
};
use crate::egui_app::SubsystemApp;
use crate::egui_app::dashboard_widgets::dashboard_scalar_value_from_pointer;
use crate::egui_app::shared_canvas_text_font_px;
use crate::model::{DashboardBinding, DashboardTargetPath, EndpointRef, Line, Port, System};
use eframe::egui::{Pos2, Rect};
use std::collections::BTreeMap;

#[test]
fn dashboard_blocks_do_not_fall_back_to_live_text() {
    assert!(!should_render_live_text(true, "PushButtonBlock"));
    assert!(!should_render_live_text(true, "ComboBox"));
    assert!(!should_render_live_text(true, "DisplayBlock"));
}

#[test]
fn non_dashboard_display_blocks_keep_live_text() {
    assert!(!should_render_live_text(true, "Gain"));
    assert!(should_render_live_text(true, "Constant"));
    assert!(should_render_live_text(true, "Display"));
    assert!(!should_render_live_text(false, "Display"));
    assert!(!should_render_live_text(true, "ManualSwitch"));
}

#[test]
fn manual_switch_live_values_map_to_expected_setting() {
    assert_eq!(manual_switch_setting_from_live_value(0.0), "0");
    assert_eq!(manual_switch_setting_from_live_value(0.49), "0");
    assert_eq!(manual_switch_setting_from_live_value(0.5), "1");
    assert_eq!(manual_switch_setting_from_live_value(1.0), "1");
}

#[test]
fn dashboard_param_source_uses_selected_vector_element_for_live_value() {
    let root = System {
        properties: indexmap::IndexMap::new(),
        blocks: Vec::new(),
        lines: Vec::new(),
        annotations: Vec::new(),
        chart: None,
    };
    let mut app = SubsystemApp::new(root, Vec::new(), BTreeMap::new(), BTreeMap::new());
    let mut block = minimal_block("Slider2", "42");
    block.block_type = "SliderBlock".to_string();
    block.dashboard_binding = Some(DashboardBinding::ParamSource {
        block_path: "Model/Slider_Vector".to_string(),
        param_name: "Value".to_string(),
        target_path: DashboardTargetPath {
            port_index: None,
            sub_path: None,
            element: None,
            element_raw_input: Some("(2)".to_string()),
        },
        uuid: "uuid-slider-2".to_string(),
    });
    app.live_values.insert(
        "uuid-slider-2".to_string(),
        crate::live_values::LiveValueEntry::new(crate::live_values::LiveValue::new(
            vec![3],
            crate::live_values::LiveValueList::Float64(vec![10.0, 20.0, 30.0]),
        )),
    );

    assert_eq!(dashboard_live_value(&app, &block), Some(20.0));
    assert_eq!(dashboard_widget_value(&app, &block), 20.0);
}

#[test]
fn dashboard_widget_value_prefers_selector_aware_binding_over_block_live_value() {
    let root = System {
        properties: indexmap::IndexMap::new(),
        blocks: Vec::new(),
        lines: Vec::new(),
        annotations: Vec::new(),
        chart: None,
    };
    let mut app = SubsystemApp::new(root, Vec::new(), BTreeMap::new(), BTreeMap::new());
    let mut block = minimal_block("Slider3", "43");
    block.block_type = "SliderBlock".to_string();
    block.dashboard_binding = Some(DashboardBinding::ParamSource {
        block_path: "Model/Slider_Vector".to_string(),
        param_name: "Value".to_string(),
        target_path: DashboardTargetPath {
            port_index: None,
            sub_path: None,
            element: None,
            element_raw_input: Some("(3)".to_string()),
        },
        uuid: "uuid-slider-3".to_string(),
    });
    app.live_values.insert(
        "uuid-slider-3".to_string(),
        crate::live_values::LiveValueEntry::new(crate::live_values::LiveValue::new(
            vec![3],
            crate::live_values::LiveValueList::Float64(vec![10.0, 20.0, 30.0]),
        )),
    );
    app.live_block_values.insert(
        app.live_value_key_for_block(&block),
        crate::live_values::LiveValueEntry::new(crate::live_values::LiveValue::new(
            vec![3],
            crate::live_values::LiveValueList::Float64(vec![10.0, 20.0, 30.0]),
        )),
    );

    assert_eq!(dashboard_widget_value(&app, &block), 30.0);
}

#[test]
fn dashboard_discrete_controls_are_editable_in_live_mode() {
    let combo = crate::model::Block {
        block_type: "ComboBox".to_string(),
        name: "Combo".to_string(),
        sid: None,
        tag_name: "Block".to_string(),
        position: None,
        zorder: None,
        commented: false,
        name_location: crate::model::NameLocation::default(),
        is_matlab_function: false,
        value: None,
        value_kind: crate::model::ValueKind::default(),
        value_rows: None,
        value_cols: None,
        properties: indexmap::IndexMap::new(),
        ref_properties: std::collections::BTreeSet::new(),
        port_counts: None,
        ports: Vec::new(),
        subsystem: None,
        system_ref: None,
        c_function: None,
        instance_data: None,
        link_data: None,
        mask: None,
        annotations: Vec::new(),
        background_color: None,
        show_name: None,
        font_size: None,
        font_weight: None,
        mask_display_text: None,
        current_setting: None,
        block_mirror: None,
        library_source: None,
        library_block_path: None,
        dashboard_binding: Some(DashboardBinding::ParamSource {
            block_path: "Model/Combo".to_string(),
            param_name: "Value".to_string(),
            target_path: DashboardTargetPath::default(),
            uuid: "uuid-combo".to_string(),
        }),
        child_order: Vec::new(),
    };
    let mut radio = combo.clone();
    radio.block_type = "RadioButtonGroup".to_string();

    assert_eq!(dashboard_input_control_kind(&combo), Some("discrete"));
    assert_eq!(dashboard_input_control_kind(&radio), Some("discrete"));
}

#[test]
fn rotary_switch_pointer_mapping_clamps_gap_and_returns_indices() {
    let mut block = crate::model::Block {
        block_type: "RotarySwitchBlock".to_string(),
        name: "Rotary".to_string(),
        sid: None,
        tag_name: "Block".to_string(),
        position: None,
        zorder: None,
        commented: false,
        name_location: crate::model::NameLocation::default(),
        is_matlab_function: false,
        value: None,
        value_kind: crate::model::ValueKind::default(),
        value_rows: None,
        value_cols: None,
        properties: indexmap::IndexMap::new(),
        ref_properties: std::collections::BTreeSet::new(),
        port_counts: None,
        ports: Vec::new(),
        subsystem: None,
        system_ref: None,
        c_function: None,
        instance_data: None,
        link_data: None,
        mask: None,
        annotations: Vec::new(),
        background_color: None,
        show_name: None,
        font_size: None,
        font_weight: None,
        mask_display_text: None,
        current_setting: None,
        block_mirror: None,
        library_source: None,
        library_block_path: None,
        dashboard_binding: None,
        child_order: Vec::new(),
    };
    block
        .properties
        .insert("Values".to_string(), "Low,Medium,High".to_string());

    let rect = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 100.0));
    let center = rect.center();

    let min_value = dashboard_scalar_value_from_pointer(
        &block,
        rect,
        Pos2::new(center.x - 10.0, rect.bottom() - 10.0),
        1.0,
    );
    let max_value = dashboard_scalar_value_from_pointer(
        &block,
        rect,
        Pos2::new(center.x + 10.0, rect.bottom() - 10.0),
        1.0,
    );
    let mid_value = dashboard_scalar_value_from_pointer(
        &block,
        rect,
        Pos2::new(center.x, rect.top() + 5.0),
        1.0,
    );

    assert_eq!(min_value, 0.0);
    assert_eq!(max_value, 2.0);
    assert_eq!(mid_value, 1.0);
}

#[test]
fn line_testpoint_follows_source_output_port() {
    let mut source = minimal_block("Source", "1");
    source.ports.push(Port {
        port_type: "out".to_string(),
        index: Some(1),
        properties: indexmap::IndexMap::from_iter([("TestPoint".to_string(), "on".to_string())]),
    });
    let sink = minimal_block("Sink", "2");
    let line = Line {
        name: None,
        zorder: None,
        src: Some(EndpointRef {
            sid: "1".to_string(),
            port_type: "out".to_string(),
            port_index: 1,
        }),
        dst: Some(EndpointRef {
            sid: "2".to_string(),
            port_type: "in".to_string(),
            port_index: 1,
        }),
        points: Vec::new(),
        labels: None,
        branches: Vec::new(),
        properties: indexmap::IndexMap::new(),
    };

    let system = System {
        properties: indexmap::IndexMap::from_iter([("Name".to_string(), "Model".to_string())]),
        blocks: vec![source, sink],
        lines: vec![line.clone()],
        annotations: Vec::new(),
        chart: None,
    };

    let resolver = ConnectionTargetResolver::new(&system);
    let targets = resolver.line_targets_for_line(&[], &line);
    assert!(line_has_testpoint(&targets));
}

#[test]
fn unnamed_line_label_stays_empty_without_explicit_line_name() {
    let mut source = minimal_block("Source Block", "1");
    source.ports.push(Port {
        port_type: "out".to_string(),
        index: Some(1),
        properties: indexmap::IndexMap::from_iter([(
            "Name".to_string(),
            "Shared Signal".to_string(),
        )]),
    });
    let sink = minimal_block("Sink", "2");
    let line = Line {
        name: None,
        zorder: None,
        src: Some(EndpointRef {
            sid: "1".to_string(),
            port_type: "out".to_string(),
            port_index: 1,
        }),
        dst: Some(EndpointRef {
            sid: "2".to_string(),
            port_type: "in".to_string(),
            port_index: 1,
        }),
        points: Vec::new(),
        labels: None,
        branches: Vec::new(),
        properties: indexmap::IndexMap::new(),
    };

    let system = System {
        properties: indexmap::IndexMap::from_iter([("Name".to_string(), "Model".to_string())]),
        blocks: vec![source, sink],
        lines: vec![line.clone()],
        annotations: Vec::new(),
        chart: None,
    };

    let resolver = ConnectionTargetResolver::new(&system);
    let targets = resolver.line_targets_for_line(&[], &line);
    assert_eq!(resolved_line_label(&line, &targets), None);
}

#[test]
fn bus_and_mux_lines_do_not_draw_propagated_fallback_names() {
    let line = Line {
        name: None,
        zorder: None,
        src: None,
        dst: None,
        points: Vec::new(),
        labels: None,
        branches: Vec::new(),
        properties: indexmap::IndexMap::new(),
    };

    let bus_targets = vec![
        ConnectionTarget {
            path: "Model/A".to_string(),
            signal_name: Some("alpha".to_string()),
            signal_names: Vec::new(),
            resolve: Some(ConnectionTargetResolve::Signal("alpha".to_string())),
            element_index: None,
            origin: ConnectionTargetOrigin::BusCreator,
            signals_only: true,
            testpoint: false,
            block_type: None,
        },
        ConnectionTarget {
            path: "Model/B".to_string(),
            signal_name: Some("beta".to_string()),
            signal_names: Vec::new(),
            resolve: Some(ConnectionTargetResolve::Signal("beta".to_string())),
            element_index: None,
            origin: ConnectionTargetOrigin::BusCreator,
            signals_only: true,
            testpoint: false,
            block_type: None,
        },
    ];
    let mux_targets = vec![ConnectionTarget {
        path: "Model/A".to_string(),
        signal_name: Some("alpha".to_string()),
        signal_names: Vec::new(),
        resolve: Some(ConnectionTargetResolve::Index(1)),
        element_index: Some(1),
        origin: ConnectionTargetOrigin::Mux,
        signals_only: true,
        testpoint: false,
        block_type: None,
    }];

    assert_eq!(resolved_line_label(&line, &bus_targets), None);
    assert_eq!(resolved_line_label(&line, &mux_targets), None);
}

#[test]
fn bus_lines_draw_thicker_than_normal_lines() {
    let normal_targets = vec![ConnectionTarget {
        path: "Model/A".to_string(),
        signal_name: Some("alpha".to_string()),
        signal_names: Vec::new(),
        resolve: Some(ConnectionTargetResolve::Signal("alpha".to_string())),
        element_index: None,
        origin: ConnectionTargetOrigin::SourceBlock,
        signals_only: true,
        testpoint: false,
        block_type: None,
    }];
    let bus_targets = vec![
        ConnectionTarget {
            path: "Model/A".to_string(),
            signal_name: Some("alpha".to_string()),
            signal_names: Vec::new(),
            resolve: Some(ConnectionTargetResolve::Signal("alpha".to_string())),
            element_index: None,
            origin: ConnectionTargetOrigin::BusCreator,
            signals_only: true,
            testpoint: false,
            block_type: None,
        },
        ConnectionTarget {
            path: "Model/B".to_string(),
            signal_name: Some("beta".to_string()),
            signal_names: Vec::new(),
            resolve: Some(ConnectionTargetResolve::Signal("beta".to_string())),
            element_index: None,
            origin: ConnectionTargetOrigin::BusCreator,
            signals_only: true,
            testpoint: false,
            block_type: None,
        },
    ];

    assert!(line_stroke_width(&bus_targets, false) > line_stroke_width(&normal_targets, false));
}

#[test]
fn line_testpoint_marker_uses_first_visible_segment() {
    let marker = line_testpoint_marker_position(&[
        Pos2::new(10.0, 10.0),
        Pos2::new(50.0, 10.0),
        Pos2::new(50.0, 40.0),
    ])
    .expect("marker position");

    assert!(marker.x > 10.0);
    assert_eq!(marker.y, 10.0);
}

#[test]
fn mask_without_display_text_falls_back_to_normal_rendering() {
    let mut block = minimal_block("Masked", "3");
    block.mask = Some(crate::model::Mask::default());

    assert!(!should_use_mask_display(&block));

    block.mask_display_text = Some(" ".to_string());
    assert!(!should_use_mask_display(&block));

    block.mask_display_text = Some("mask label".to_string());
    assert!(should_use_mask_display(&block));
}

#[test]
fn aux_label_font_scaling_uses_name_controls() {
    let scaled = shared_canvas_text_font_px(0.5, 2.0);
    assert_eq!(scaled, 32.0);

    let scaled_down = shared_canvas_text_font_px(0.5, 0.5);
    assert_eq!(scaled_down, 8.0);
}

fn minimal_block(name: &str, sid: &str) -> crate::model::Block {
    crate::model::Block {
        block_type: "Constant".to_string(),
        name: name.to_string(),
        sid: Some(sid.to_string()),
        tag_name: "Block".to_string(),
        position: None,
        zorder: None,
        commented: false,
        name_location: crate::model::NameLocation::default(),
        is_matlab_function: false,
        value: None,
        value_kind: crate::model::ValueKind::default(),
        value_rows: None,
        value_cols: None,
        properties: indexmap::IndexMap::new(),
        ref_properties: std::collections::BTreeSet::new(),
        port_counts: None,
        ports: Vec::new(),
        subsystem: None,
        system_ref: None,
        c_function: None,
        instance_data: None,
        link_data: None,
        mask: None,
        annotations: Vec::new(),
        background_color: None,
        show_name: None,
        font_size: None,
        font_weight: None,
        mask_display_text: None,
        current_setting: None,
        block_mirror: None,
        library_source: None,
        library_block_path: None,
        dashboard_binding: None,
        child_order: Vec::new(),
    }
}
