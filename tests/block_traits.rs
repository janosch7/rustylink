//! Tests for the feature-independent block-type trait table.

use rustylink::simulink_libraries::traits::{
    KNOWN_BLOCK_TYPES, SignalRole, block_traits, is_known_block_type,
};

#[test]
fn known_block_types_are_sorted_for_binary_search() {
    let mut sorted = KNOWN_BLOCK_TYPES.to_vec();
    sorted.sort_unstable();
    assert_eq!(sorted, KNOWN_BLOCK_TYPES);
    assert!(is_known_block_type("Gain"));
    assert!(!is_known_block_type("NotABlock"));
}

#[test]
fn roles_describe_signal_behaviour() {
    assert_eq!(block_traits("Mux").signal_role, SignalRole::Mux);
    assert!(block_traits("Reference").path_prefix_matched);
    assert!(!block_traits("SubSystem").path_prefix_matched);
    assert!(SignalRole::Container.crosses_system_boundary());
    assert!(!SignalRole::Mux.crosses_system_boundary());
    assert!(!SignalRole::Plain.propagates_local_metadata());
}
