#![cfg(feature = "egui")]

use eframe::egui::Color32;
use rustylink::egui_app::ui::colors::hsv_to_color32;
use rustylink::egui_app::ui::line_coloring::{
    assign_line_colors, circular_dist, compute_line_adjacency, hue_to_color32, rel_luminance,
};

#[test]
fn circular_dist_basic() {
    assert!((circular_dist(0.0, 0.5) - 0.5).abs() < 1e-6);
    assert!((circular_dist(0.1, 0.9) - 0.2).abs() < 1e-6);
    assert!((circular_dist(0.3, 0.3) - 0.0).abs() < 1e-6);
}

#[test]
fn hsv_red_green_blue() {
    let red = hsv_to_color32(0.0, 1.0, 1.0);
    assert_eq!(red.r(), 255);
    assert_eq!(red.g(), 0);
    assert_eq!(red.b(), 0);

    let green = hsv_to_color32(1.0 / 3.0, 1.0, 1.0);
    assert_eq!(green.g(), 255);

    let blue = hsv_to_color32(2.0 / 3.0, 1.0, 1.0);
    assert_eq!(blue.b(), 255);
}

#[test]
fn rel_luminance_white_black() {
    let white_lum = rel_luminance(Color32::WHITE);
    assert!((white_lum - 1.0).abs() < 0.01);
    let black_lum = rel_luminance(Color32::BLACK);
    assert!(black_lum < 0.01);
}

#[test]
fn empty_lines_returns_empty() {
    let adj = compute_line_adjacency(&[]);
    assert!(adj.is_empty());
    let colors = assign_line_colors(&adj, 0.9);
    assert!(colors.is_empty());
}

#[test]
fn single_line_assigned_color() {
    let adj = vec![vec![]]; // 1 line, no neighbors
    let colors = assign_line_colors(&adj, 0.9);
    assert_eq!(colors.len(), 1);
}

#[test]
fn adjacent_lines_get_different_hues() {
    // Two lines adjacent to each other
    let adj = vec![vec![1], vec![0]];
    let colors = assign_line_colors(&adj, 0.9);
    assert_eq!(colors.len(), 2);
    // They should be different
    assert_ne!(colors[0], colors[1]);
}

#[test]
fn many_lines_all_get_colors() {
    let n = 20;
    let adj: Vec<Vec<usize>> = (0..n)
        .map(|i| (0..n).filter(|&j| j != i && (i + j) % 3 == 0).collect())
        .collect();
    let colors = assign_line_colors(&adj, 0.9);
    assert_eq!(colors.len(), n);
}

#[test]
fn hue_to_color32_vivid() {
    // hue_to_color32 should produce saturated, bright colors
    let c = hue_to_color32(0.0);
    // Red-ish at hue 0
    assert!(c.r() > 200);
}

#[test]
fn circular_dist_wraps_around() {
    // 0.1 and 0.9 are 0.2 apart on the circle, not 0.8
    let d = circular_dist(0.1, 0.9);
    assert!((d - 0.2).abs() < 0.001);
}

#[test]
fn circular_dist_same_point() {
    assert!((circular_dist(0.5, 0.5)).abs() < f32::EPSILON);
}

#[test]
fn adjacent_colors_avoid_dark_on_dark_bg() {
    // With high bg luminance (light BG), assigned colors should be dark enough
    let adj = vec![vec![1], vec![0]];
    let colors = assign_line_colors(&adj, 0.95);
    for c in &colors {
        // Should not be near-white on near-white bg
        let lum = rel_luminance(*c);
        assert!(lum < 0.85, "Color too bright for light bg: lum={}", lum);
    }
}
