#![cfg(feature = "egui")]

use eframe::egui::{Pos2, Rect, Vec2};
use rustylink::egui_app::ui::view_transform::{preview_block_rect, shared_canvas_text_font_px, ViewTransform};
use rustylink::egui_app::state::ViewerDragState;

fn make_transform() -> ViewTransform {
    let bb = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 100.0));
    let avail = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(500.0, 500.0));
    ViewTransform::new(bb, avail, 20.0, 1.0, Vec2::ZERO)
}

#[test]
fn to_screen_from_screen_roundtrip() {
    let vt = make_transform();
    let model_pt = Pos2::new(50.0, 25.0);
    let screen_pt = vt.to_screen(model_pt);
    let back = vt.from_screen(screen_pt);
    assert!(
        (back.x - model_pt.x).abs() < 0.01,
        "x: {} vs {}",
        back.x,
        model_pt.x
    );
    assert!(
        (back.y - model_pt.y).abs() < 0.01,
        "y: {} vs {}",
        back.y,
        model_pt.y
    );
}

#[test]
fn origin_maps_to_margin() {
    let vt = make_transform();
    let screen = vt.to_screen(Pos2::new(0.0, 0.0));
    assert!((screen.x - 20.0).abs() < 0.01);
    assert!((screen.y - 20.0).abs() < 0.01);
}

#[test]
fn zoom_at_preserves_cursor_position() {
    let vt = make_transform();
    let cursor = Pos2::new(250.0, 250.0);
    let world_before = vt.from_screen(cursor);
    let (new_zoom, new_pan) = vt.zoom_at(cursor, 1.5);
    let vt2 = ViewTransform {
        zoom: new_zoom,
        pan: new_pan,
        ..vt
    };
    let world_after = vt2.from_screen(cursor);
    assert!((world_before.x - world_after.x).abs() < 0.5);
    assert!((world_before.y - world_after.y).abs() < 0.5);
}

#[test]
fn font_scale_positive_at_min_zoom() {
    let vt = ViewTransform::new(
        Rect::from_min_max(Pos2::ZERO, Pos2::new(100.0, 100.0)),
        Rect::from_min_max(Pos2::ZERO, Pos2::new(500.0, 500.0)),
        20.0,
        0.2,
        Vec2::ZERO,
    );
    assert!(vt.font_scale() > 0.0);
}

#[test]
fn shared_canvas_text_font_px_is_continuous() {
    let just_below = shared_canvas_text_font_px(0.37, 0.85);
    let just_above = shared_canvas_text_font_px(0.38, 0.85);
    assert!(just_above > just_below);
    assert!((just_above - just_below) < 1.0);
}

#[test]
fn shared_canvas_text_font_px_matches_default_label_basis() {
    let px = shared_canvas_text_font_px(0.5, 0.85);
    assert!((px - 13.6).abs() < f32::EPSILON);
}

#[test]
fn preview_block_rect_no_drag() {
    let r = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(50.0, 30.0));
    let sids = std::collections::BTreeSet::new();
    let result = preview_block_rect(&ViewerDragState::None, &sids, Some("1"), r);
    assert_eq!(result, r);
}

#[test]
fn preview_block_rect_blocks_drag() {
    let r = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(50.0, 30.0));
    let mut sids = std::collections::BTreeSet::new();
    sids.insert("1".to_string());
    let state = ViewerDragState::Blocks {
        current_dx: 10,
        current_dy: -5,
    };
    let result = preview_block_rect(&state, &sids, Some("1"), r);
    assert!((result.left() - 10.0).abs() < 0.01);
}
