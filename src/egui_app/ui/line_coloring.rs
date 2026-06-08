//! Graph-coloring algorithm for assigning visually distinct colors to signal lines.
//!
//! This module extracts the per-frame line color computation from the monolithic
//! `update_internal` so it can be cached and unit-tested independently.

use eframe::egui::Color32;
use std::collections::HashMap;

use super::colors::hsv_to_color32;

/// Re-export luminance from colors module as `rel_luminance` (legacy alias).
pub use super::colors::luminance as rel_luminance;

/// Minimum circular distance between two hue values on the [0, 1) circle.
pub fn circular_dist(a: f32, b: f32) -> f32 {
    let d = (a - b).abs();
    d.min(1.0 - d)
}

/// Convert a hue value to a vivid line color (s=0.85, v=0.95).
pub fn hue_to_color32(h: f32) -> Color32 {
    hsv_to_color32(h, 0.85, 0.95)
}

/// Build a per-line adjacency list from the signal endpoint graph.
///
/// Two lines are adjacent if they share a block SID (source, destination, or
/// any branch destination).
pub fn compute_line_adjacency(lines: &[crate::model::Line]) -> Vec<Vec<usize>> {
    let n = lines.len();
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut sid_to_lines: HashMap<String, Vec<usize>> = HashMap::new();

    for (i, l) in lines.iter().enumerate() {
        if let Some(src) = &l.src {
            sid_to_lines.entry(src.sid.clone()).or_default().push(i);
        }
        if let Some(dst) = &l.dst {
            sid_to_lines.entry(dst.sid.clone()).or_default().push(i);
        }
        fn collect_branch_sids(br: &crate::model::Branch, out: &mut Vec<String>) {
            if let Some(dst) = &br.dst {
                out.push(dst.sid.clone());
            }
            for sub in &br.branches {
                collect_branch_sids(sub, out);
            }
        }
        let mut br_sids: Vec<String> = Vec::new();
        for br in &l.branches {
            collect_branch_sids(br, &mut br_sids);
        }
        for sid in br_sids {
            sid_to_lines.entry(sid).or_default().push(i);
        }
    }

    for idxs in sid_to_lines.values() {
        for a in 0..idxs.len() {
            for b in (a + 1)..idxs.len() {
                let i = idxs[a];
                let j = idxs[b];
                if !adjacency[i].contains(&j) {
                    adjacency[i].push(j);
                }
                if !adjacency[j].contains(&i) {
                    adjacency[j].push(i);
                }
            }
        }
    }

    adjacency
}

/// Assign visually distinct colors to each line using a greedy graph-coloring
/// approach that maximises hue distance between adjacent lines.
///
/// Returns one `Color32` per line (indexed by line position).
pub fn assign_line_colors(adjacency: &[Vec<usize>], background_luminance: f32) -> Vec<Color32> {
    let n = adjacency.len();
    if n == 0 {
        return Vec::new();
    }

    let sample_count = (n * 8).max(64);
    let mut candidates: Vec<f32> = (0..sample_count)
        .map(|i| (i as f32) / (sample_count as f32))
        .collect();

    let max_lum = (background_luminance - 0.25).clamp(0.0, 1.0);
    candidates.retain(|&h| rel_luminance(hue_to_color32(h)) <= max_lum);
    if candidates.is_empty() {
        candidates = (0..sample_count)
            .map(|i| (i as f32) / (sample_count as f32))
            .collect();
    }

    // Process lines from most-connected to least-connected.
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&i| (-(adjacency[i].len() as isize), i as isize));

    let mut assigned_hues: Vec<Option<f32>> = vec![None; n];
    let mut remaining: Vec<f32> = candidates.clone();

    for i in order {
        let neigh_hues: Vec<f32> = adjacency[i]
            .iter()
            .filter_map(|&j| assigned_hues[j])
            .collect();

        let mut best_h = 0.0;
        let mut best_score = -1.0f32;
        for &h in &remaining {
            let used: Vec<f32> = if neigh_hues.is_empty() {
                assigned_hues.iter().flatten().copied().collect()
            } else {
                neigh_hues.clone()
            };
            let score: f32 = if used.is_empty() {
                1.0
            } else {
                used.iter()
                    .map(|&u| circular_dist(h, u))
                    .fold(1.0, |a, d| f32::min(a, d))
            };
            if score > best_score || (score == best_score && h < best_h) {
                best_score = score;
                best_h = h;
            }
        }
        assigned_hues[i] = Some(best_h);
        if let Some(pos) = remaining
            .iter()
            .position(|&x| (x - best_h).abs() < f32::EPSILON)
        {
            remaining.remove(pos);
        }
    }

    assigned_hues
        .into_iter()
        .enumerate()
        .map(|(i, h)| {
            let default_h = (i as f32) / (n.max(1) as f32);
            let c = hue_to_color32(h.unwrap_or(default_h));
            if rel_luminance(c) > max_lum {
                hsv_to_color32(h.unwrap_or(default_h), 0.85, 0.75)
            } else {
                c
            }
        })
        .collect()
}

