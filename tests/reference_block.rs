use anyhow::Result;
use camino::Utf8PathBuf;
use rustylink::parser::{ContentSource, SimulinkParser};
use std::collections::HashMap;

struct MemSource {
    files: HashMap<String, String>,
}
impl ContentSource for MemSource {
    fn read_to_string(&mut self, path: &camino::Utf8Path) -> Result<String> {
        self.files
            .get(path.as_str())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("not found: {}", path))
    }
    fn list_dir(&mut self, path: &camino::Utf8Path) -> Result<Vec<Utf8PathBuf>> {
        let prefix = path.as_str().trim_end_matches('/').to_string() + "/";
        let mut out = Vec::new();
        for k in self.files.keys() {
            if k.starts_with(&prefix) {
                out.push(Utf8PathBuf::from(k.clone()));
            }
        }
        Ok(out)
    }
}

#[test]
fn parse_reference_tag_as_block() {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<System>
  <Reference Name="Logic" SID="53">
    <P Name="Position">[325, 264, 425, 306]</P>
    <P Name="ZOrder">377</P>
    <P Name="LibraryVersion">1.1</P>
    <P Name="SourceBlock">ASXTestLibrary/Logic</P>
    <P Name="SourceType">SubSystem</P>
    <PortProperties>
      <Port Type="out" Index="1">
        <P Name="Name">result</P>
        <P Name="TestPoint">on</P>
      </Port>
    </PortProperties>
  </Reference>
</System>
"#;

    let path = Utf8PathBuf::from("mem://reference_test.xml");
    let mut files = HashMap::new();
    files.insert(path.as_str().to_string(), xml.to_string());
    let source = MemSource { files };
    let mut parser = SimulinkParser::new("/", source);
    let system = parser.parse_system_file(&path).expect("parse system XML");

    assert_eq!(system.blocks.len(), 1);
    let b = &system.blocks[0];
    assert_eq!(b.name, "Logic");
    assert_eq!(b.sid.as_deref(), Some("53"));
    // Tag <Reference> should be treated as a Reference block
    assert_eq!(b.block_type, "Reference");
    assert_eq!(b.ports.len(), 1);
    assert_eq!(b.ports[0].port_type, "out");
}

#[test]
fn check_verification_reference_blocks_have_no_output_port() {
    // A Check/Verification Reference block with `<PortCounts in="1"/>` must
    // not gain a spurious output port from the virtual-library stub merge.
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<System>
  <Block BlockType="Reference" Name="Check Static Gap" SID="276">
    <P Name="Position">[-185, 2627, -125, 2673]</P>
    <P Name="ZOrder">100000</P>
    <P Name="SourceBlock">simulink/Model Verification/Check Static Gap</P>
    <P Name="SourceType">Checks_SGap</P>
    <PortCounts in="1" />
  </Block>
  <Block BlockType="Reference" Name="Check Dynamic Range" SID="275">
    <P Name="Position">[-185, 2700, -125, 2746]</P>
    <P Name="ZOrder">100001</P>
    <P Name="SourceBlock">simulink/Model Verification/Check Dynamic Range</P>
    <P Name="SourceType">Checks_DRange</P>
    <PortCounts in="3" />
  </Block>
</System>
"#;

    let path = Utf8PathBuf::from("mem://check_test.xml");
    let mut files = HashMap::new();
    files.insert(path.as_str().to_string(), xml.to_string());
    let source = MemSource { files };
    let mut parser = SimulinkParser::new("/", source);
    let system = parser.parse_system_file(&path).expect("parse system XML");
    SimulinkParser::<MemSource>::resolve_library_references(&mut system.clone(), &[])
        .expect("resolve library references");

    assert_eq!(system.blocks.len(), 2);

    // Check Static Gap: 1 input, no output
    let gap = &system.blocks[0];
    assert_eq!(gap.name, "Check Static Gap");
    let pc = gap.port_counts.as_ref().expect("has PortCounts");
    assert_eq!(pc.ins, Some(1));
    assert_eq!(
        pc.outs, None,
        "Check Static Gap must not have an output port"
    );

    // Check Dynamic Range: 3 inputs, no output
    let range = &system.blocks[1];
    assert_eq!(range.name, "Check Dynamic Range");
    let pc = range.port_counts.as_ref().expect("has PortCounts");
    assert_eq!(pc.ins, Some(3));
    assert_eq!(
        pc.outs, None,
        "Check Dynamic Range must not have an output port"
    );
}

#[test]
fn inport_shadow_block_parses_and_renders_like_inport() {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<System>
  <Block BlockType="InportShadow" Name="joint_ref_bus_" SID="221298">
    <P Name="Position">[375, 537, 410, 553]</P>
    <P Name="ZOrder">6634</P>
    <P Name="ForegroundColor">red</P>
    <P Name="Port">2</P>
    <PortProperties>
      <Port Type="out" Index="1">
        <P Name="PropagatedSignals">joint_ref_bus</P>
      </Port>
    </PortProperties>
  </Block>
</System>
"#;

    let path = Utf8PathBuf::from("mem://inport_shadow_test.xml");
    let mut files = HashMap::new();
    files.insert(path.as_str().to_string(), xml.to_string());
    let source = MemSource { files };
    let mut parser = SimulinkParser::new("/", source);
    let system = parser.parse_system_file(&path).expect("parse system XML");

    assert_eq!(system.blocks.len(), 1);
    let b = &system.blocks[0];
    assert_eq!(b.block_type, "InportShadow");
    assert_eq!(b.name, "joint_ref_bus_");
    assert_eq!(b.sid.as_deref(), Some("221298"));
    // Port property is parsed
    assert_eq!(b.properties.get("Port").map(|s| s.as_str()), Some("2"));
    // One output port (the shadow feeds signals into the subsystem)
    assert_eq!(b.ports.len(), 1);
    assert_eq!(b.ports[0].port_type, "out");
    assert_eq!(b.ports[0].index, Some(1));
    // PropagatedSignals is preserved on the port
    assert_eq!(
        b.ports[0]
            .properties
            .get("PropagatedSignals")
            .map(|s| s.as_str()),
        Some("joint_ref_bus")
    );
}

#[cfg(feature = "egui")]
#[test]
fn unresolvable_library_link_is_labelled_with_the_library_name() {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<System>
  <Reference Name="Logic" SID="53">
    <P Name="Position">[325, 264, 425, 306]</P>
    <P Name="SourceBlock">ASXTestLibrary/Logic</P>
    <P Name="SourceType">SubSystem</P>
  </Reference>
</System>
"#;

    let path = Utf8PathBuf::from("mem://missing_library.xml");
    let mut files = HashMap::new();
    files.insert(path.as_str().to_string(), xml.to_string());
    let source = MemSource { files };
    let mut parser = SimulinkParser::new("/", source);
    let mut system = parser.parse_system_file(&path).expect("parse system XML");
    SimulinkParser::<MemSource>::resolve_library_references(&mut system, &[])
        .expect("resolve library references");

    let b = &system.blocks[0];
    assert!(b.library_missing, "library cannot be located");
    assert_eq!(
        rustylink::simulink_libraries::labels::library_link(b).as_deref(),
        Some("ASXTestLibrary\n(not found)")
    );
}

#[cfg(feature = "egui")]
#[test]
fn resolved_library_link_shows_the_library_and_its_port_names() {
    let host = r#"<?xml version="1.0" encoding="utf-8"?>
<System>
  <Reference Name="Logic" SID="53">
    <P Name="Position">[325, 264, 425, 306]</P>
    <P Name="SourceBlock">ASXTestLibrary/Logic</P>
    <P Name="SourceType">SubSystem</P>
  </Reference>
</System>
"#;
    // The system the library's `Logic` block contains; the parser copies it onto
    // the host block once the library is located.
    let library_contents = r#"<?xml version="1.0" encoding="utf-8"?>
<System>
  <Block BlockType="Inport" Name="u" SID="1">
    <P Name="Position">[20, 20, 40, 40]</P>
    <P Name="Port">1</P>
  </Block>
  <Block BlockType="Outport" Name="result" SID="2">
    <P Name="Position">[120, 20, 140, 40]</P>
    <P Name="Port">1</P>
  </Block>
</System>
"#;

    let host_path = Utf8PathBuf::from("mem://host.xml");
    let lib_path = Utf8PathBuf::from("mem://library_logic.xml");
    let mut files = HashMap::new();
    files.insert(host_path.as_str().to_string(), host.to_string());
    files.insert(lib_path.as_str().to_string(), library_contents.to_string());
    let source = MemSource { files };
    let mut parser = SimulinkParser::new("/", source);
    let mut system = parser.parse_system_file(&host_path).expect("parse host");
    let contents = parser.parse_system_file(&lib_path).expect("parse library");

    let b = &mut system.blocks[0];
    b.subsystem = Some(Box::new(contents));
    b.library_source = Some("ASXTestLibrary".to_string());
    b.library_block_path = Some("ASXTestLibrary/Logic".to_string());

    let b = &system.blocks[0];
    assert_eq!(
        rustylink::simulink_libraries::labels::library_link(b).as_deref(),
        Some("ASXTestLibrary")
    );
    let cfg = rustylink::egui_app::get_block_type_cfg(b);
    assert_eq!(
        rustylink::egui_app::port_label_display_name(b, 1, true, &cfg),
        "u"
    );
    assert_eq!(
        rustylink::egui_app::port_label_display_name(b, 1, false, &cfg),
        "result"
    );
}
