#![cfg(feature = "egui")]

use indexmap::IndexMap;
use rustylink::egui_app::state::ComputedViewCache;
use rustylink::model::{Block, NameLocation, System, ValueKind};

fn empty_system() -> System {
    System {
        properties: IndexMap::new(),
        blocks: Vec::new(),
        lines: Vec::new(),
        annotations: Vec::new(),
        chart: None,
    }
}

fn test_block(name: &str, sid: Option<&str>) -> Block {
    Block {
        block_type: "Gain".to_string(),
        name: name.to_string(),
        sid: sid.map(str::to_string),
        tag_name: "Block".to_string(),
        position: None,
        zorder: None,
        commented: false,
        name_location: NameLocation::default(),
        is_matlab_function: false,
        value: None,
        value_kind: ValueKind::default(),
        value_rows: None,
        value_cols: None,
        properties: IndexMap::new(),
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

fn subsystem_block(name: &str, sid: Option<&str>, subsystem: System) -> Block {
    let mut block = test_block(name, sid);
    block.block_type = "SubSystem".to_string();
    block.subsystem = Some(Box::new(subsystem));
    block
}

fn root_with_subsystem() -> System {
    System {
        properties: IndexMap::new(),
        blocks: vec![subsystem_block("Sub", Some("1"), empty_system())],
        lines: Vec::new(),
        annotations: Vec::new(),
        chart: None,
    }
}

#[test]
fn cache_starts_invalid() {
    let cache = ComputedViewCache::default();
    assert!(!cache.is_valid(&[], cache.generation));
    assert!(!cache.is_valid(&["Root".to_string()], cache.generation));
}

#[test]
fn cache_valid_after_mark() {
    let mut cache = ComputedViewCache::default();
    let path = vec!["Root".to_string()];
    cache.mark_valid(&path, cache.generation);
    assert!(cache.is_valid(&path, cache.generation));
}

#[test]
fn cache_invalid_after_invalidate() {
    let mut cache = ComputedViewCache::default();
    let path = vec!["Root".to_string()];
    cache.mark_valid(&path, cache.generation);
    assert!(cache.is_valid(&path, cache.generation));
    cache.invalidate();
    assert!(!cache.is_valid(&path, cache.generation));
}

#[test]
fn resolver_reused_until_topology_changes() {
    use std::sync::Arc;

    let mut cache = ComputedViewCache::default();
    let mut root = root_with_subsystem();

    let first = cache.ensure_resolver(&root);
    let again = cache.ensure_resolver(&root);
    assert!(
        Arc::ptr_eq(&first, &again),
        "resolver should be reused when the model is unchanged"
    );

    // A geometry-only edit (block Position) must reuse the cached resolver.
    root.blocks[0]
        .properties
        .insert("Position".to_string(), "[1, 2, 3, 4]".to_string());
    let after_move = cache.ensure_resolver(&root);
    assert!(
        Arc::ptr_eq(&first, &after_move),
        "layout-only edits must not rebuild the resolver"
    );

    // A topology edit (adding a block) must rebuild the resolver.
    root.blocks.push(test_block("Extra", Some("2")));
    let after_topology = cache.ensure_resolver(&root);
    assert!(
        !Arc::ptr_eq(&first, &after_topology),
        "topology edits must rebuild the resolver"
    );
}
