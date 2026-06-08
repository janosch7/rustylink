use rustylink::live_values::{
    LiveValue, LiveValueDisplayOptions, LiveValueEntry, LiveValueList,
    scalar_requires_scientific, scalar_requires_scientific_with_bounds,
};

#[test]
fn formats_vector_with_spaces_and_default_decimals() {
    let text = LiveValueEntry::new(LiveValue::new(
        vec![2],
        LiveValueList::Float64(vec![1.234, 5.678]),
    ))
    .formatted_text();

    assert_eq!(text, "1.23, 5.68");
}

#[test]
fn formats_scientific_when_needed() {
    let text = LiveValueEntry::new(LiveValue::new(
        vec![2],
        LiveValueList::Float64(vec![1.1e-5, 2.0e5]),
    ))
    .formatted_text();

    assert_eq!(text, "1.10e-5, 2.00e5");
}

#[test]
fn scientific_thresholds_match_expected_bounds() {
    assert!(!scalar_requires_scientific(0.1));
    assert!(scalar_requires_scientific(0.099));
    assert!(!scalar_requires_scientific(1.0e4));
    assert!(scalar_requires_scientific(1.0e4 + 1.0));
}

#[test]
fn formats_matrix_in_column_major_simulink_order() {
    let text = LiveValueEntry::new(LiveValue::new(
        vec![2, 2],
        LiveValueList::Int32(vec![1, 2, 3, 4]),
    ))
    .formatted_text();

    assert_eq!(text, "1, 3;2, 4");
}

#[test]
fn first_f64_reads_numeric_scalars() {
    let entry = LiveValueEntry::new(LiveValue::new(vec![], LiveValueList::UInt16(vec![7])));
    assert_eq!(entry.first_f64(), Some(7.0));
}

#[test]
fn explicit_display_options_override_defaults() {
    let entry = LiveValueEntry::new(LiveValue::new(
        vec![1],
        LiveValueList::Float64(vec![1.23456]),
    ))
    .with_display(LiveValueDisplayOptions {
        float_decimals: 4,
        scientific_lower_bound: rustylink::live_values::LIVE_SCIENTIFIC_LOWER_BOUND,
        scientific_upper_bound: rustylink::live_values::LIVE_SCIENTIFIC_UPPER_BOUND,
        always_scientific: false,
    });

    assert_eq!(entry.formatted_text(), "1.2346");
}

#[test]
fn custom_scientific_bounds_are_used() {
    let entry = LiveValueEntry::new(LiveValue::new(
        vec![2],
        LiveValueList::Float64(vec![0.5, 20.0]),
    ))
    .with_display(LiveValueDisplayOptions {
        float_decimals: 2,
        scientific_lower_bound: 1.0,
        scientific_upper_bound: 10.0,
        always_scientific: false,
    });

    assert_eq!(entry.formatted_text(), "5.00e-1, 2.00e1");
    assert!(scalar_requires_scientific_with_bounds(0.5, 1.0, 10.0));
    assert!(scalar_requires_scientific_with_bounds(20.0, 1.0, 10.0));
    assert!(!scalar_requires_scientific_with_bounds(5.0, 1.0, 10.0));
}

#[test]
fn always_scientific_overrides_thresholds() {
    let entry =
        LiveValueEntry::new(LiveValue::new(vec![1], LiveValueList::Float64(vec![12.34])))
            .with_display(LiveValueDisplayOptions {
                float_decimals: 2,
                scientific_lower_bound: 1.0,
                scientific_upper_bound: 100.0,
                always_scientific: true,
            });

    assert_eq!(entry.formatted_text(), "1.23e1");
}
