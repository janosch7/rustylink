use eframe::egui::Color32;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn luminance(c: Color32) -> f32 {
    fn to_lin(u: u8) -> f32 {
        let s = (u as f32) / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * to_lin(c.r()) + 0.7152 * to_lin(c.g()) + 0.0722 * to_lin(c.b())
}

pub fn contrast_color(bg: Color32) -> Color32 {
    let bg_lum = luminance(bg);
    let dark = Color32::from_rgb(25, 35, 45);
    let light = Color32::from_rgb(235, 245, 245);
    // Pick whichever foreground yields the higher WCAG contrast ratio against
    // the background, so mid-tone block colors (e.g. salmon Gain blocks) get
    // legible dark text instead of a faint light glyph.
    let ratio = |fg: Color32| {
        let fg_lum = luminance(fg);
        let (hi, lo) = if fg_lum > bg_lum {
            (fg_lum, bg_lum)
        } else {
            (bg_lum, fg_lum)
        };
        (hi + 0.05) / (lo + 0.05)
    };
    if ratio(dark) >= ratio(light) {
        dark
    } else {
        light
    }
}

pub fn hsv_to_color32(h: f32, s: f32, v: f32) -> Color32 {
    let h6 = (h * 6.0) % 6.0;
    let c = v * s;
    let x = c * (1.0 - ((h6 % 2.0) - 1.0).abs());
    let (r1, g1, b1) = if h6 < 1.0 {
        (c, x, 0.0)
    } else if h6 < 2.0 {
        (x, c, 0.0)
    } else if h6 < 3.0 {
        (0.0, c, x)
    } else if h6 < 4.0 {
        (0.0, x, c)
    } else if h6 < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let m = v - c;
    let (r, g, b) = (r1 + m, g1 + m, b1 + m);
    Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

pub fn hash_color(input: &str, s: f32, v: f32) -> Color32 {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = hasher.finish();
    let h = (hash as f32 / u64::MAX as f32) % 1.0;
    hsv_to_color32(h, s, v)
}

/// Parse a Simulink model color string into a [`Color32`].
///
/// Handles the three encodings that appear in `.slx` models: named colors
/// (`"blue"`), `#rrggbb` hex, and MATLAB-style fractional RGB triplets
/// (`"[0.90, 0.90, 1.0]"`, each component in `0.0..=1.0`).
pub fn parse_model_color(raw: &str) -> Option<Color32> {
    let s = raw.trim();
    match s.to_lowercase().as_str() {
        "yellow" => return Some(Color32::from_rgb(255, 230, 120)),
        "red" => return Some(Color32::from_rgb(230, 90, 90)),
        "green" => return Some(Color32::from_rgb(120, 210, 140)),
        "blue" => return Some(Color32::from_rgb(100, 160, 230)),
        "black" => return Some(Color32::from_rgb(40, 40, 40)),
        "white" => return Some(Color32::from_rgb(235, 235, 235)),
        "gray" | "grey" => return Some(Color32::from_rgb(180, 180, 180)),
        _ => {}
    }
    let lower = s.to_lowercase();
    if lower.starts_with('#')
        && lower.len() == 7
        && let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&lower[1..3], 16),
            u8::from_str_radix(&lower[3..5], 16),
            u8::from_str_radix(&lower[5..7], 16),
        )
    {
        return Some(Color32::from_rgb(r, g, b));
    }
    if let Some(inner) = s.strip_prefix('[').and_then(|t| t.strip_suffix(']')) {
        let parts: Vec<f32> = inner
            .split([',', ' '])
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .filter_map(|t| t.parse::<f32>().ok())
            .collect();
        if parts.len() >= 3 {
            let to8 = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            return Some(Color32::from_rgb(
                to8(parts[0]),
                to8(parts[1]),
                to8(parts[2]),
            ));
        }
    }
    None
}

pub fn block_base_color(
    block: &crate::model::Block,
    cfg: &crate::block_types::BlockTypeConfig,
) -> Color32 {
    if let Some(ref color_str) = block.background_color
        && let Some(c) = parse_model_color(color_str)
    {
        return c;
    }
    if let Some(bg) = cfg.background {
        return Color32::from_rgb(bg.0, bg.1, bg.2);
    }
    hash_color(&block.block_type, 0.35, 0.90)
}

/// Neutral block fill used in "less colorful" mode.  A light gray in both
/// themes; kept slightly lighter in dark mode so the body stays visible.
pub fn monochrome_block_fill(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(200, 202, 208)
    } else {
        Color32::from_rgb(224, 226, 230)
    }
}

/// Border for blocks drawn in "less colorful" mode, so a light-gray body stays
/// distinguishable from the canvas (especially in light themes).
pub fn monochrome_block_border(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(150, 152, 158)
    } else {
        Color32::from_rgb(120, 122, 130)
    }
}

/// Neutral signal-line color used in "less colorful" mode.
pub fn monochrome_line_color(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(170, 172, 178)
    } else {
        Color32::from_rgb(105, 107, 115)
    }
}

/// The model-defined fill color of an area annotation, if this annotation is an
/// area (`AnnotationType == "area_annotation"`) that carries a `BackgroundColor`.
pub fn area_annotation_fill(a: &crate::model::Annotation) -> Option<Color32> {
    if a.properties.get("AnnotationType").map(String::as_str) != Some("area_annotation") {
        return None;
    }
    a.properties
        .get("BackgroundColor")
        .and_then(|c| parse_model_color(c))
}

/// The model-defined border color of an area annotation, falling back to a
/// slightly darkened version of its fill.
pub fn area_annotation_border(a: &crate::model::Annotation) -> Option<Color32> {
    if a.properties.get("AnnotationType").map(String::as_str) != Some("area_annotation") {
        return None;
    }
    if let Some(fg) = a
        .properties
        .get("ForegroundColor")
        .and_then(|c| parse_model_color(c))
    {
        return Some(fg);
    }
    area_annotation_fill(a).map(|c| {
        let d = |v: u8| (v as f32 * 0.7).round() as u8;
        Color32::from_rgb(d(c.r()), d(c.g()), d(c.b()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Annotation;
    use indexmap::IndexMap;

    #[test]
    fn parse_model_color_named_hex_and_bracket() {
        assert_eq!(
            parse_model_color("blue"),
            Some(Color32::from_rgb(100, 160, 230))
        );
        assert_eq!(
            parse_model_color("#ff8000"),
            Some(Color32::from_rgb(255, 128, 0))
        );
        // MATLAB fractional RGB triplet (the encoding used by area annotations).
        assert_eq!(
            parse_model_color("[0.901961, 0.901961, 1.000000]"),
            Some(Color32::from_rgb(230, 230, 255))
        );
        assert_eq!(parse_model_color("not a color"), None);
    }

    fn area_annotation(props: &[(&str, &str)]) -> Annotation {
        let mut properties = IndexMap::new();
        for (k, v) in props {
            properties.insert((*k).to_string(), (*v).to_string());
        }
        Annotation {
            sid: None,
            text: None,
            position: None,
            zorder: None,
            interpreter: None,
            properties,
        }
    }

    #[test]
    fn area_fill_only_for_area_annotations() {
        let area = area_annotation(&[
            ("AnnotationType", "area_annotation"),
            ("BackgroundColor", "[0.0, 0.5, 1.0]"),
        ]);
        assert_eq!(
            area_annotation_fill(&area),
            Some(Color32::from_rgb(0, 128, 255))
        );

        // Plain text annotations (no area type) never get a background fill.
        let text = area_annotation(&[("BackgroundColor", "[0.0, 0.5, 1.0]")]);
        assert_eq!(area_annotation_fill(&text), None);
    }
}
