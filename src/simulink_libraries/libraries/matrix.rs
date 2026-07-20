//! Matrix-operations library (`matrix_library`).
//!
//! These blocks carry dedicated SVG icons shipped under `icons/matrix/`.  Port
//! counts mirror [`crate::simulink_libraries::stubs::MATRIX_BLOCKS`], which the
//! core parser uses to synthesise stubs when the `.slx` library file is absent.

#![cfg(feature = "egui")]

use crate::simulink_libraries::types::{IOPorts, SimulinkBlockDefinition, SimulinkIcon};

const CAT: &str = "Matrix Operations";

const fn svg(path: &'static str) -> SimulinkIcon {
    SimulinkIcon::Svg(path)
}

/// Typeset-math icon (superscript); see [`crate::egui_app::render::draw_math_icon`].
const fn math(spec: &'static str) -> SimulinkIcon {
    SimulinkIcon::Math(spec)
}

pub static BLOCKS: &[SimulinkBlockDefinition] = &[
    SimulinkBlockDefinition::new("Identity Matrix", CAT)
        .with_aliases(&["IdentityMatrix"])
        .with_description("Generate an identity matrix")
        .with_ports(IOPorts::None, IOPorts::Fixed(1))
        .with_icon(svg("matrix/identity_matrix.svg")),
    SimulinkBlockDefinition::new("Is Triangular", CAT)
        .with_aliases(&["IsTriangular"])
        .with_description("Test whether a matrix is triangular")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(svg("matrix/is_triangular.svg")),
    SimulinkBlockDefinition::new("Is Symmetric", CAT)
        .with_aliases(&["IsSymmetric"])
        .with_description("Test whether a matrix is symmetric")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(svg("matrix/is_symmetric.svg")),
    SimulinkBlockDefinition::new("Cross Product", CAT)
        .with_description("Cross product of two vectors")
        .with_ports(IOPorts::Fixed(2), IOPorts::Fixed(1))
        .with_icon(svg("matrix/cross_product.svg")),
    SimulinkBlockDefinition::new("Matrix Multiply", CAT)
        .with_description("Matrix multiplication")
        .with_ports(IOPorts::Variable(2), IOPorts::Fixed(1))
        .with_icon(svg("matrix/matrix_product.svg")),
    SimulinkBlockDefinition::new("Submatrix", CAT)
        .with_description("Select a submatrix")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(svg("matrix/submatrix.svg")),
    SimulinkBlockDefinition::new("Transpose", CAT)
        .with_description("Transpose a matrix")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(math("sup:A^T")),
    SimulinkBlockDefinition::new("Hermitian Transpose", CAT)
        .with_description("Complex-conjugate (Hermitian) transpose")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(math("sup:A^H")),
    SimulinkBlockDefinition::new("Matrix Square", CAT)
        .with_aliases(&["Square"])
        .with_description("Square a matrix (A*A)")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(svg("matrix/matrix_square.svg")),
    SimulinkBlockDefinition::new("Permute Matrix", CAT)
        .with_aliases(&["Permute Columns", "PermuteMatrix", "PermuteColumns"])
        .with_description("Permute rows or columns of a matrix")
        .with_ports(IOPorts::Fixed(2), IOPorts::Fixed(1)),
    SimulinkBlockDefinition::new("Extract Diagonal", CAT)
        .with_aliases(&["ExtractDiag"])
        .with_description("Extract the main diagonal of a matrix")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(svg("matrix/extract_diagonal.svg")),
    SimulinkBlockDefinition::new("Create Diagonal Matrix", CAT)
        .with_aliases(&["DiagonalMatrix"])
        .with_description("Create a diagonal matrix from a vector")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(svg("matrix/create_diagonal_matrix.svg")),
    SimulinkBlockDefinition::new("Expand Scalar", CAT)
        .with_aliases(&["ExpandScalar"])
        .with_description("Expand a scalar to a matrix")
        .with_ports(IOPorts::Fixed(1), IOPorts::Fixed(1))
        .with_icon(svg("matrix/expand_scalar_to_matrix.svg")),
    SimulinkBlockDefinition::new("Matrix Concatenate", CAT)
        .with_description("Concatenate matrices")
        .with_ports(IOPorts::Variable(2), IOPorts::Fixed(1)),
];
