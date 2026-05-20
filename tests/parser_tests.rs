use camino::Utf8Path;
use indexmap::IndexMap;
use roxmltree::Document;
use rustylink::block::{parse_block_shallow, parse_line_node};
use rustylink::builtin_libraries::matrix_library;
use rustylink::model::System;
use rustylink::parser::{FsSource, SimulinkParser, is_virtual_library};

use rustylink::parser::helpers::clean_whitespace;

#[test]
fn virtual_library_detection() {
    assert!(is_virtual_library("simulink"));
    assert!(is_virtual_library("Simulink.SLX"));
    assert!(is_virtual_library("matrix_library"));
    assert!(is_virtual_library("simulink/Logic and Bit Operations"));
    assert!(is_virtual_library("simulink/Logic and Bit"));
    assert!(is_virtual_library("Simulink/logic and BIT"));
    assert!(is_virtual_library("simulink/Discrete"));
    assert!(!is_virtual_library("other"));
}

#[test]
fn resolving_virtual_library_does_not_error() {
    // Build a system containing a single block referencing the virtual lib
    let mut blk = rustylink::editor::operations::create_default_block("Some", "B", 0, 0, 0, 0);
    blk.properties.insert(
        "SourceBlock".to_string(),
        "simulink/Logic and Bit/Foo".to_string(),
    );
    let mut sys = System {
        properties: IndexMap::new(),
        blocks: vec![blk],
        lines: Vec::new(),
        annotations: Vec::new(),
        chart: None,
    };

    // Call the public resolver; should succeed without panicking or error.
    SimulinkParser::<FsSource>::resolve_library_references(&mut sys, &[]).unwrap();
    // The block still exists and has received a stub from the simulink/*
    // virtual-library fallback.
    assert_eq!(sys.blocks.len(), 1);
    assert_eq!(
        sys.blocks[0].library_source.as_deref(),
        Some("simulink/Logic and Bit")
    );
    assert_eq!(
        sys.blocks[0].library_block_path.as_deref(),
        Some("simulink/Logic and Bit/Foo")
    );
}

#[test]
fn matrix_library_helpers_work() {
    // name recognition
    assert!(matrix_library::is_matrix_library_name("matrix_library"));
    assert!(matrix_library::is_matrix_library_name(
        "Matrix_Library/Thing"
    ));
    assert!(!matrix_library::is_matrix_library_name("other"));

    // port counts for known and unknown names
    // CamelCase names are transparently mapped to the spaced canonical name
    // via humanize_camel_case (e.g. "IdentityMatrix" → "Identity Matrix").
    assert_eq!(matrix_library::port_counts_for("IdentityMatrix"), (0, 1));
    // All-lowercase with no space can't be mapped to a spaced name, so
    // "crossproduct" is no longer recognised (falls back to the default).
    assert_eq!(matrix_library::port_counts_for("crossproduct"), (1, 1));
    assert_eq!(matrix_library::port_counts_for("unknown"), (1, 1));

    // whitespace collapse: multiple spaces are treated the same as a single
    // space, but spaces are not removed entirely.
    let a = matrix_library::port_counts_for("Cross   Product");
    let b = matrix_library::port_counts_for("Cross Product");
    assert_eq!(a, b);
    // CamelCase "CrossProduct" is humanized to "Cross Product" and matches.
    assert_eq!(matrix_library::port_counts_for("CrossProduct"), b);

    // block list uses the spaced canonical name
    assert!(
        matrix_library::BLOCKS
            .iter()
            .any(|b| b.name == "Identity Matrix")
    );

    // stub creation produces a block with the expected fields
    let stub = matrix_library::create_stub("Foo");
    assert_eq!(stub.block_type, "Foo");
    assert_eq!(stub.ports.len(), 2); // default 1 in + 1 out
}

#[test]
fn clean_whitespace_basic() {
    assert_eq!(clean_whitespace("foo"), "foo");
    assert_eq!(clean_whitespace("  foo  "), "foo");
    assert_eq!(clean_whitespace("foo   bar"), "foo bar");
    assert_eq!(clean_whitespace("foo\nbar\tbaz"), "foo bar baz");
    assert_eq!(
        clean_whitespace("   multiple   \n whitespace  "),
        "multiple whitespace"
    );
}

#[test]
fn parser_strips_newlines_from_block_and_port_signal_names() {
    let doc = Document::parse(
        r#"<Block BlockType="ComplexToRealImag" Name="Complex to&#10;Real-Imag" SID="1">
            <PortProperties>
                <Port Type="out" Index="1">
                    <P Name="Name">sig&#10;name</P>
                    <P Name="PropagatedSignals">prop&#10;sig</P>
                </Port>
            </PortProperties>
        </Block>"#,
    )
    .expect("block xml");

    let block =
        parse_block_shallow(doc.root_element(), Utf8Path::new(".")).expect("parse block shallow");

    assert_eq!(block.name, "Complex to Real-Imag");
    assert_eq!(block.ports.len(), 1);
    assert_eq!(
        block.ports[0].properties.get("Name").map(String::as_str),
        Some("sig name")
    );
    assert_eq!(
        block.ports[0]
            .properties
            .get("PropagatedSignals")
            .map(String::as_str),
        Some("prop sig")
    );
}

#[test]
fn parser_strips_newlines_from_line_names() {
    let doc = Document::parse(
        r#"<Line>
            <P Name="Name">my&#10;signal</P>
            <P Name="Src">1#out:1</P>
            <P Name="Dst">2#in:1</P>
        </Line>"#,
    )
    .expect("line xml");

    let line = parse_line_node(doc.root_element()).expect("parse line");

    assert_eq!(line.name.as_deref(), Some("my signal"));
    assert_eq!(
        line.properties.get("Name").map(String::as_str),
        Some("my signal")
    );
}

#[test]
fn parser_assigns_missing_port_indices_per_port_type() {
    let doc = Document::parse(
        r#"<Block BlockType="ComplexToRealImag" Name="ComplexToRealImag" SID="1">
            <PortProperties>
                <Port Type="in">
                    <P Name="Name">input</P>
                </Port>
                <Port Type="out">
                    <P Name="Name">real</P>
                </Port>
                <Port Type="out">
                    <P Name="Name">imag</P>
                </Port>
            </PortProperties>
        </Block>"#,
    )
    .expect("block xml");

    let block =
        parse_block_shallow(doc.root_element(), Utf8Path::new(".")).expect("parse block shallow");

    assert_eq!(block.ports.len(), 3);
    assert_eq!(block.ports[0].port_type, "in");
    assert_eq!(block.ports[0].index, Some(1));
    assert_eq!(block.ports[1].port_type, "out");
    assert_eq!(block.ports[1].index, Some(1));
    assert_eq!(block.ports[2].port_type, "out");
    assert_eq!(block.ports[2].index, Some(2));
}
