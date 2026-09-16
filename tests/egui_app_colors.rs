//! Tests for Simulink color parsing and annotation fills.

use eframe::egui::Color32;
use indexmap::IndexMap;
use rustylink::egui_app::ui::colors::{area_annotation_fill, parse_model_color};
use rustylink::model::Annotation;

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
