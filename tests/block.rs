use rustylink::block::parse_array_property_value;

#[test]
fn numeric_array_properties_preserve_multiple_cells() {
    let xml = r#"
        <Array PropName="States" Type="Cell" Dimension="1*2">
          <Cell Class="double">[0.0, 1.0]</Cell>
          <Cell Class="double">Matrix(2,3)
[[100.0, 212.0, 19.0]; [0.0, 114.0, 189.0]]</Cell>
        </Array>
    "#;
    let doc = roxmltree::Document::parse(xml).unwrap();
    let value = parse_array_property_value(doc.root_element()).unwrap();
    assert_eq!(
        value,
        "[0.0, 1.0]|Matrix(2,3)\n[[100.0, 212.0, 19.0]; [0.0, 114.0, 189.0]]"
    );
}
