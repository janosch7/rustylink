#![cfg(feature = "egui")]

use indexmap::IndexMap;
use rustylink::egui_app::{get_block_type_cfg, port_label_display_name};
use rustylink::model::System;

#[test]
fn port_labels_do_not_fall_back_to_propagated_signals() {
    let mut block =
        rustylink::editor::operations::create_default_block("SubSystem", "SubSystem", 0, 0, 1, 1);
    block.ports = vec![rustylink::model::Port {
        port_type: "out".to_string(),
        index: Some(1),
        properties: IndexMap::from_iter([(
            "PropagatedSignals".to_string(),
            "ConnectedSignal".to_string(),
        )]),
    }];

    let cfg = get_block_type_cfg(&block);
    assert_eq!(port_label_display_name(&block, 1, false, &cfg), "Out1");
}

#[test]
fn fixed_port_labels_ignore_port_name_overrides() {
    let mut block = rustylink::editor::operations::create_default_block(
        "ComplexToRealImag",
        "ComplexToRealImag",
        0,
        0,
        1,
        2,
    );
    block.ports = vec![
        rustylink::model::Port {
            port_type: "in".to_string(),
            index: Some(1),
            properties: Default::default(),
        },
        rustylink::model::Port {
            port_type: "out".to_string(),
            index: Some(1),
            properties: IndexMap::from_iter([("Name".to_string(), "VisibleSignal".to_string())]),
        },
        rustylink::model::Port {
            port_type: "out".to_string(),
            index: Some(2),
            properties: IndexMap::from_iter([("Name".to_string(), "OtherSignal".to_string())]),
        },
    ];

    let cfg = get_block_type_cfg(&block);
    assert_eq!(port_label_display_name(&block, 1, true, &cfg), "Re+Im");
    assert_eq!(port_label_display_name(&block, 1, false, &cfg), "Re");
    assert_eq!(port_label_display_name(&block, 2, false, &cfg), "Im");
}

#[test]
fn subsystem_port_labels_use_internal_boundary_block_names() {
    let mut block =
        rustylink::editor::operations::create_default_block("SubSystem", "SubSystem", 0, 0, 1, 1);
    block.subsystem = Some(Box::new(System {
        properties: IndexMap::new(),
        blocks: vec![
            rustylink::model::Block {
                name: "SubsystemInput".to_string(),
                properties: IndexMap::from_iter([("Port".to_string(), "1".to_string())]),
                ports: vec![],
                block_type: "Inport".to_string(),
                sid: Some("10".to_string()),
                tag_name: "Block".to_string(),
                position: None,
                zorder: None,
                commented: false,
                name_location: rustylink::model::NameLocation::Bottom,
                is_matlab_function: false,
                value: None,
                value_kind: rustylink::model::ValueKind::default(),
                value_rows: None,
                value_cols: None,
                ref_properties: Default::default(),
                port_counts: None,
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
            },
            rustylink::model::Block {
                name: "SubsystemOutput".to_string(),
                properties: IndexMap::from_iter([("Port".to_string(), "1".to_string())]),
                ports: vec![],
                block_type: "Outport".to_string(),
                sid: Some("11".to_string()),
                tag_name: "Block".to_string(),
                position: None,
                zorder: None,
                commented: false,
                name_location: rustylink::model::NameLocation::Bottom,
                is_matlab_function: false,
                value: None,
                value_kind: rustylink::model::ValueKind::default(),
                value_rows: None,
                value_cols: None,
                ref_properties: Default::default(),
                port_counts: None,
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
            },
        ],
        lines: Vec::new(),
        annotations: Vec::new(),
        chart: None,
    }));

    let cfg = get_block_type_cfg(&block);
    assert_eq!(
        port_label_display_name(&block, 1, true, &cfg),
        "SubsystemInput"
    );
    assert_eq!(
        port_label_display_name(&block, 1, false, &cfg),
        "SubsystemOutput"
    );
}
