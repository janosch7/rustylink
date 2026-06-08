#![cfg(feature = "egui")]

use eframe::egui::Pos2;
use indexmap::IndexMap;
use rustylink::egui_app::ui::signal_routing::{
    collect_branch_handle_positions, get_branch_mut, move_branch_point, move_line_layout,
    move_line_point, move_branch_layouts, orthogonalize_polyline, push_orthogonal_segments,
    register_endpoint, register_branch_endpoints, compute_port_info,
};
use rustylink::model::{Branch, Line, Point};

/// Helper to create a minimal `Line` for tests.
fn test_line(points: Vec<Point>, branches: Vec<Branch>) -> Line {
    Line {
        src: None,
        dst: None,
        name: None,
        zorder: None,
        labels: None,
        properties: IndexMap::new(),
        points,
        branches,
    }
}

/// Helper to create a minimal `Branch` for tests.
fn test_branch(points: Vec<Point>, branches: Vec<Branch>) -> Branch {
    Branch {
        dst: None,
        name: None,
        zorder: None,
        labels: None,
        properties: IndexMap::new(),
        points,
        branches,
    }
}

#[test]
fn orthogonalize_empty() {
    assert!(orthogonalize_polyline(&[]).is_empty());
}

#[test]
fn orthogonalize_single_point() {
    let pts = vec![Pos2::new(10.0, 20.0)];
    let result = orthogonalize_polyline(&pts);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], Pos2::new(10.0, 20.0));
}

#[test]
fn orthogonalize_horizontal_stays() {
    let pts = vec![Pos2::new(0.0, 5.0), Pos2::new(10.0, 5.0)];
    let result = orthogonalize_polyline(&pts);
    assert_eq!(result.len(), 2);
}

#[test]
fn orthogonalize_vertical_stays() {
    let pts = vec![Pos2::new(5.0, 0.0), Pos2::new(5.0, 10.0)];
    let result = orthogonalize_polyline(&pts);
    assert_eq!(result.len(), 2);
}

#[test]
fn orthogonalize_diagonal_inserts_corner() {
    let pts = vec![Pos2::new(0.0, 0.0), Pos2::new(10.0, 10.0)];
    let result = orthogonalize_polyline(&pts);
    assert_eq!(result.len(), 3);
    // Corner inserted horizontal-first: (10, 0) then (10, 10)
    assert_eq!(result[1], Pos2::new(10.0, 0.0));
    assert_eq!(result[2], Pos2::new(10.0, 10.0));
}

#[test]
fn orthogonalize_multiple_diagonals() {
    let pts = vec![
        Pos2::new(0.0, 0.0),
        Pos2::new(10.0, 5.0),
        Pos2::new(20.0, 15.0),
    ];
    let result = orthogonalize_polyline(&pts);
    // Each diagonal adds a corner point
    assert!(result.len() >= 5);
    // All segments should be axis-aligned
    for seg in result.windows(2) {
        let dx = (seg[0].x - seg[1].x).abs();
        let dy = (seg[0].y - seg[1].y).abs();
        assert!(
            dx < f32::EPSILON || dy < f32::EPSILON,
            "Non-orthogonal segment: {:?} -> {:?}",
            seg[0],
            seg[1]
        );
    }
}

#[test]
fn push_segments_from_polyline() {
    let pts = vec![
        Pos2::new(0.0, 0.0),
        Pos2::new(10.0, 0.0),
        Pos2::new(10.0, 10.0),
    ];
    let mut segs = Vec::new();
    push_orthogonal_segments(&pts, &mut segs);
    assert_eq!(segs.len(), 2);
    assert_eq!(segs[0], (Pos2::new(0.0, 0.0), Pos2::new(10.0, 0.0)));
    assert_eq!(segs[1], (Pos2::new(10.0, 0.0), Pos2::new(10.0, 10.0)));
}

#[test]
fn move_line_point_compensates_next() {
    let mut line = test_line(
        vec![
            Point { x: 10, y: 0 },
            Point { x: 20, y: 0 },
            Point { x: 0, y: 30 },
        ],
        vec![],
    );
    move_line_point(&mut line, 1, 5, -3);
    assert_eq!(line.points[1].x, 25);
    assert_eq!(line.points[1].y, -3);
    // Next point compensated
    assert_eq!(line.points[2].x, -5);
    assert_eq!(line.points[2].y, 33);
    // Previous point unchanged
    assert_eq!(line.points[0].x, 10);
    assert_eq!(line.points[0].y, 0);
}

#[test]
fn move_line_point_last_index_no_crash() {
    let mut line = test_line(vec![Point { x: 5, y: 5 }], vec![]);
    // Moving the last (only) point shouldn't panic
    move_line_point(&mut line, 0, 3, 3);
    assert_eq!(line.points[0].x, 8);
    assert_eq!(line.points[0].y, 8);
}

#[test]
fn move_branch_point_basic() {
    let mut branch = test_branch(vec![Point { x: 10, y: 10 }, Point { x: 20, y: 20 }], vec![]);
    move_branch_point(&mut branch, 0, 5, -5);
    assert_eq!(branch.points[0].x, 15);
    assert_eq!(branch.points[0].y, 5);
    assert_eq!(branch.points[1].x, 15);
    assert_eq!(branch.points[1].y, 25);
}

#[test]
fn move_line_layout_shifts_all() {
    let mut line = test_line(
        vec![Point { x: 0, y: 0 }, Point { x: 10, y: 10 }],
        vec![test_branch(vec![Point { x: 5, y: 5 }], vec![])],
    );
    move_line_layout(&mut line, 3, -2);
    assert_eq!(line.points[0], Point { x: 3, y: -2 });
    assert_eq!(line.points[1], Point { x: 13, y: 8 });
    assert_eq!(line.branches[0].points[0], Point { x: 8, y: 3 });
}

#[test]
fn get_branch_mut_navigates_tree() {
    let mut branches = vec![
        test_branch(
            vec![Point { x: 1, y: 1 }],
            vec![test_branch(vec![Point { x: 2, y: 2 }], vec![])],
        ),
        test_branch(vec![Point { x: 3, y: 3 }], vec![]),
    ];

    // Navigate to first branch
    let b = get_branch_mut(&mut branches, &[0]).unwrap();
    assert_eq!(b.points[0], Point { x: 1, y: 1 });

    // Navigate to nested branch
    let b = get_branch_mut(&mut branches, &[0, 0]).unwrap();
    assert_eq!(b.points[0], Point { x: 2, y: 2 });

    // Navigate to second top-level branch
    let b = get_branch_mut(&mut branches, &[1]).unwrap();
    assert_eq!(b.points[0], Point { x: 3, y: 3 });

    // Invalid path returns None
    assert!(get_branch_mut(&mut branches, &[5]).is_none());
    assert!(get_branch_mut(&mut branches, &[0, 5]).is_none());
}

#[test]
fn collect_branch_handles_basic() {
    let identity = |p: Pos2| p;
    let branches = vec![test_branch(
        vec![Point { x: 10, y: 0 }, Point { x: 0, y: 20 }],
        vec![],
    )];
    let mut out = Vec::new();
    collect_branch_handle_positions(
        Pos2::new(0.0, 0.0),
        &branches,
        &identity,
        &mut Vec::new(),
        &mut out,
    );
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].0, vec![0]); // branch path
    assert_eq!(out[0].1, 0); // point index
    assert_eq!(out[0].2, Pos2::new(10.0, 0.0));
    assert_eq!(out[1].2, Pos2::new(10.0, 20.0));
}
