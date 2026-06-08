//! Pure-function corner manipulation utilities for signal lines.
//!
//! These functions operate on the `Vec<Point>` model (cumulative relative
//! offsets from the source endpoint) without any egui dependency, making them
//! straightforward to unit-test.

use crate::model::{Line, Point};

// ---------------------------------------------------------------------------
// Insert / Remove / Merge corners
// ---------------------------------------------------------------------------

/// Insert a new corner point at `index` in the line's point list.
///
/// The inserted point gets the given `offset`. The *next* point (if any)
/// is adjusted so that all downstream geometry remains unchanged.
pub fn insert_corner(points: &mut Vec<Point>, index: usize, offset: Point) {
    if index > points.len() {
        return;
    }
    // If inserting before an existing point, subtract our offset from it
    // so the cumulative position after index stays the same.
    if index < points.len() {
        let next = &mut points[index];
        next.x -= offset.x;
        next.y -= offset.y;
    }
    points.insert(index, offset);
}

/// Remove the corner point at `index`, merging its offset into the next
/// point so downstream geometry is preserved.
///
/// Returns the removed point (for undo support), or `None` if the index
/// was out of range.
pub fn remove_corner(points: &mut Vec<Point>, index: usize) -> Option<Point> {
    if index >= points.len() {
        return None;
    }
    let removed = points.remove(index);
    // Compensate the next point so the cumulative position doesn't change.
    if index < points.len() {
        points[index].x += removed.x;
        points[index].y += removed.y;
    }
    Some(removed)
}

/// Merge adjacent corner points that are closer than `threshold` model
/// units apart (Manhattan distance). The second point is absorbed into
/// the first, preserving downstream positions.
#[allow(dead_code)]
pub fn merge_adjacent_corners(points: &mut Vec<Point>, threshold: i32) {
    let mut i = 0;
    while i + 1 < points.len() {
        let dist = points[i].x.abs() + points[i].y.abs();
        if dist <= threshold {
            // Merge point[i] into point[i+1]
            points[i + 1].x += points[i].x;
            points[i + 1].y += points[i].y;
            points.remove(i);
            // Don't advance i — check the merged result against the next
        } else {
            i += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Auto-adjust on block move
// ---------------------------------------------------------------------------

/// Adjust a line's corners when a connected block moves by `(dx, dy)`.
///
/// * `is_source = true`  → the *source* block moved; adjust/insert the
///   **first** point so the line tracks the block.
/// * `is_source = false` → the *destination* block moved; adjust/insert the
///   **last** point so the line tracks the destination.
///
/// When the line already has points, the first (or last) point is adjusted.
/// When the line has *no* points, a new point is created to absorb the delta.
pub fn auto_adjust_on_block_move(line: &mut Line, is_source: bool, dx: i32, dy: i32) {
    if dx == 0 && dy == 0 {
        return;
    }
    if is_source {
        // Source block moved — the source anchor moves with the block.
        // Offset of first point relative to the (now-moved) source must
        // be *reduced* by the block's delta to keep the absolute position
        // of the first corner unchanged.
        if let Some(first) = line.points.first_mut() {
            first.x -= dx;
            first.y -= dy;
        } else {
            // No corners: insert one to absorb the discrepancy.
            line.points.push(Point { x: -dx, y: -dy });
        }
    } else {
        // Destination block moved — the destination anchor moves with the
        // block. We need to extend/adjust the last point to reach the new
        // destination position.
        if let Some(last) = line.points.last_mut() {
            last.x += dx;
            last.y += dy;
        } else {
            line.points.push(Point { x: dx, y: dy });
        }
    }
}

/// Adjust all branches whose destination matches the moved block.
pub fn auto_adjust_branches_on_block_move(
    branches: &mut [crate::model::Branch],
    moved_sid: &str,
    dx: i32,
    dy: i32,
) {
    for branch in branches.iter_mut() {
        if let Some(dst) = &branch.dst {
            if dst.sid == moved_sid {
                if let Some(last) = branch.points.last_mut() {
                    last.x += dx;
                    last.y += dy;
                } else {
                    branch.points.push(Point { x: dx, y: dy });
                }
            }
        }
        auto_adjust_branches_on_block_move(&mut branch.branches, moved_sid, dx, dy);
    }
}

// ---------------------------------------------------------------------------
// Orthogonal enforcement (model-level)
// ---------------------------------------------------------------------------

/// Snap each point's offset so the segment from its predecessor is either
/// purely horizontal or purely vertical (horizontal-first convention).
///
/// This operates on the *relative offsets* (the `Point` values), not
/// absolute screen positions.
#[allow(dead_code)]
pub fn enforce_orthogonal(points: &mut Vec<Point>) {
    for point in points.iter_mut() {
        if point.x != 0 && point.y != 0 {
            // Diagonal offset — split into horizontal-only.
            // The vertical component will be absorbed by the next
            // orthogonalization pass (or the final segment to the
            // destination). For now, zero out the smaller axis.
            if point.x.abs() >= point.y.abs() {
                point.y = 0;
            } else {
                point.x = 0;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

