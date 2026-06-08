#![cfg(feature = "egui")]

use indexmap::IndexMap;
use rustylink::egui_app::ui::corner_ops::{
    auto_adjust_branches_on_block_move, auto_adjust_on_block_move, enforce_orthogonal,
    insert_corner, merge_adjacent_corners, remove_corner,
};
use rustylink::model::{Branch, EndpointRef, Line, Point};

fn test_line(points: Vec<Point>) -> Line {
    Line {
        src: None,
        dst: None,
        name: None,
        zorder: None,
        labels: None,
        properties: IndexMap::new(),
        points,
        branches: vec![],
    }
}

#[test]
fn insert_corner_at_beginning() {
    let mut pts = vec![Point { x: 20, y: 0 }, Point { x: 0, y: 30 }];
    insert_corner(&mut pts, 0, Point { x: 10, y: 0 });
    assert_eq!(pts.len(), 3);
    assert_eq!(pts[0], Point { x: 10, y: 0 });
    assert_eq!(pts[1], Point { x: 10, y: 0 });
    assert_eq!(pts[2], Point { x: 0, y: 30 });
}

#[test]
fn insert_corner_at_middle() {
    let mut pts = vec![Point { x: 30, y: 0 }, Point { x: 0, y: 20 }];
    insert_corner(&mut pts, 1, Point { x: 10, y: 5 });
    assert_eq!(pts.len(), 3);
    assert_eq!(pts[0], Point { x: 30, y: 0 });
    assert_eq!(pts[1], Point { x: 10, y: 5 });
    assert_eq!(pts[2], Point { x: -10, y: 15 });
}

#[test]
fn insert_corner_at_end() {
    let mut pts = vec![Point { x: 10, y: 0 }];
    insert_corner(&mut pts, 1, Point { x: 5, y: 5 });
    assert_eq!(pts.len(), 2);
    assert_eq!(pts[0], Point { x: 10, y: 0 });
    assert_eq!(pts[1], Point { x: 5, y: 5 });
}

#[test]
fn insert_corner_into_empty() {
    let mut pts = Vec::new();
    insert_corner(&mut pts, 0, Point { x: 10, y: 0 });
    assert_eq!(pts.len(), 1);
    assert_eq!(pts[0], Point { x: 10, y: 0 });
}

#[test]
fn remove_corner_merges_offset() {
    let mut pts = vec![
        Point { x: 10, y: 0 },
        Point { x: 5, y: 5 },
        Point { x: 0, y: 20 },
    ];
    let removed = remove_corner(&mut pts, 1);
    assert_eq!(removed, Some(Point { x: 5, y: 5 }));
    assert_eq!(pts.len(), 2);
    assert_eq!(pts[0], Point { x: 10, y: 0 });
    assert_eq!(pts[1], Point { x: 5, y: 25 });
}

#[test]
fn remove_corner_last_point() {
    let mut pts = vec![Point { x: 10, y: 0 }, Point { x: 5, y: 5 }];
    let removed = remove_corner(&mut pts, 1);
    assert_eq!(removed, Some(Point { x: 5, y: 5 }));
    assert_eq!(pts.len(), 1);
    assert_eq!(pts[0], Point { x: 10, y: 0 });
}

#[test]
fn remove_corner_only_point() {
    let mut pts = vec![Point { x: 10, y: 5 }];
    let removed = remove_corner(&mut pts, 0);
    assert_eq!(removed, Some(Point { x: 10, y: 5 }));
    assert!(pts.is_empty());
}

#[test]
fn remove_corner_out_of_range() {
    let mut pts = vec![Point { x: 1, y: 2 }];
    assert_eq!(remove_corner(&mut pts, 5), None);
    assert_eq!(pts.len(), 1);
}

#[test]
fn merge_adjacent_within_threshold() {
    let mut pts = vec![
        Point { x: 2, y: 0 },
        Point { x: 20, y: 0 },
        Point { x: 1, y: 1 },
        Point { x: 0, y: 30 },
    ];
    merge_adjacent_corners(&mut pts, 5);
    assert_eq!(pts.len(), 2);
    assert_eq!(pts[0], Point { x: 22, y: 0 });
    assert_eq!(pts[1], Point { x: 1, y: 31 });
}

#[test]
fn merge_adjacent_nothing_to_merge() {
    let mut pts = vec![Point { x: 10, y: 0 }, Point { x: 0, y: 20 }];
    merge_adjacent_corners(&mut pts, 5);
    assert_eq!(pts.len(), 2);
}

#[test]
fn auto_adjust_source_with_corners() {
    let mut line = test_line(vec![Point { x: 20, y: 0 }, Point { x: 0, y: 30 }]);
    auto_adjust_on_block_move(&mut line, true, 5, 3);
    assert_eq!(line.points[0], Point { x: 15, y: -3 });
    assert_eq!(line.points[1], Point { x: 0, y: 30 });
}

#[test]
fn auto_adjust_source_no_corners() {
    let mut line = test_line(vec![]);
    auto_adjust_on_block_move(&mut line, true, 5, 3);
    assert_eq!(line.points.len(), 1);
    assert_eq!(line.points[0], Point { x: -5, y: -3 });
}

#[test]
fn auto_adjust_dest_with_corners() {
    let mut line = test_line(vec![Point { x: 20, y: 0 }, Point { x: 0, y: 30 }]);
    auto_adjust_on_block_move(&mut line, false, 5, 3);
    assert_eq!(line.points[0], Point { x: 20, y: 0 });
    assert_eq!(line.points[1], Point { x: 5, y: 33 });
}

#[test]
fn auto_adjust_dest_no_corners() {
    let mut line = test_line(vec![]);
    auto_adjust_on_block_move(&mut line, false, 5, 3);
    assert_eq!(line.points.len(), 1);
    assert_eq!(line.points[0], Point { x: 5, y: 3 });
}

#[test]
fn auto_adjust_zero_delta_noop() {
    let mut line = test_line(vec![Point { x: 10, y: 0 }]);
    auto_adjust_on_block_move(&mut line, true, 0, 0);
    assert_eq!(line.points[0], Point { x: 10, y: 0 });
}

#[test]
fn auto_adjust_branch_dest() {
    let mut branches = vec![Branch {
        dst: Some(EndpointRef {
            sid: "42".to_string(),
            port_type: "in".to_string(),
            port_index: 1,
        }),
        name: None,
        zorder: None,
        labels: None,
        properties: IndexMap::new(),
        points: vec![Point { x: 10, y: 0 }, Point { x: 0, y: 20 }],
        branches: vec![],
    }];
    auto_adjust_branches_on_block_move(&mut branches, "42", 3, -2);
    assert_eq!(branches[0].points[1], Point { x: 3, y: 18 });
}

#[test]
fn auto_adjust_branch_no_match() {
    let mut branches = vec![Branch {
        dst: Some(EndpointRef {
            sid: "99".to_string(),
            port_type: "in".to_string(),
            port_index: 1,
        }),
        name: None,
        zorder: None,
        labels: None,
        properties: IndexMap::new(),
        points: vec![Point { x: 10, y: 0 }],
        branches: vec![],
    }];
    auto_adjust_branches_on_block_move(&mut branches, "42", 3, -2);
    assert_eq!(branches[0].points[0], Point { x: 10, y: 0 });
}

#[test]
fn enforce_orthogonal_horizontal_dominant() {
    let mut pts = vec![Point { x: 10, y: 3 }];
    enforce_orthogonal(&mut pts);
    assert_eq!(pts[0], Point { x: 10, y: 0 });
}

#[test]
fn enforce_orthogonal_vertical_dominant() {
    let mut pts = vec![Point { x: 2, y: 15 }];
    enforce_orthogonal(&mut pts);
    assert_eq!(pts[0], Point { x: 0, y: 15 });
}

#[test]
fn enforce_orthogonal_already_axis_aligned() {
    let mut pts = vec![Point { x: 10, y: 0 }, Point { x: 0, y: 20 }];
    enforce_orthogonal(&mut pts);
    assert_eq!(pts[0], Point { x: 10, y: 0 });
    assert_eq!(pts[1], Point { x: 0, y: 20 });
}

#[test]
fn enforce_orthogonal_empty() {
    let mut pts: Vec<Point> = vec![];
    enforce_orthogonal(&mut pts);
    assert!(pts.is_empty());
}

#[test]
fn insert_then_remove_roundtrip() {
    let original = vec![Point { x: 20, y: 0 }, Point { x: 0, y: 30 }];
    let mut pts = original.clone();
    insert_corner(&mut pts, 1, Point { x: 10, y: 5 });
    assert_eq!(pts.len(), 3);
    let removed = remove_corner(&mut pts, 1);
    assert_eq!(removed, Some(Point { x: 10, y: 5 }));
    assert_eq!(pts, original);
}
