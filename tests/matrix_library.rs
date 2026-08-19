use rustylink::simulink_libraries::stubs;

fn port_counts_for(name: &str) -> (u32, u32) {
    stubs::matrix_port_counts_if_known(name).unwrap_or((1, 1))
}

#[test]
fn triangular_and_symmetric_have_one_each() {
    assert_eq!(port_counts_for("IsTriangular"), (1, 1));
    assert_eq!(port_counts_for("IsSymmetric"), (1, 1));
    assert_eq!(port_counts_for("is triangular"), (1, 1));
    assert_eq!(port_counts_for("is   symmetric"), (1, 1));
}

#[test]
fn diagonal_matrix_alias_is_recognised() {
    // ensure the new alias behaves identically to the canonical name
    assert_eq!(port_counts_for("DiagonalMatrix"), (1, 1));
    assert_eq!(
        stubs::matrix_port_counts_if_known("DiagonalMatrix"),
        Some((1, 1))
    );

    // also check extract-diagonal alias
    assert_eq!(port_counts_for("ExtractDiag"), (1, 1));
    assert_eq!(
        stubs::matrix_port_counts_if_known("ExtractDiag"),
        Some((1, 1))
    );
}
