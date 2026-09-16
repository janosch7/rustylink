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
        // Return all files that reside under the given logical directory prefix
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
fn parse_chart_and_mapping_then_open_matlab_function() {
    // Minimal system containing a subsystem referencing system_18 which doesn't exist as XML in MemSource
    let sys_root = r#"<?xml version="1.0" encoding="utf-8"?>
<System>
  <Block BlockType="SubSystem" Name="Wall clock" SID="18">
    <P Name="SFBlockType">MATLAB Function</P>
    <System Ref="system_18"/>
  </Block>
  <!-- The referenced system_18 is intentionally missing as a file; parser should tolerate this. -->
</System>
"#;

    // Chart XML based on provided sample
    let chart_18 = r#"<?xml version="1.0" encoding="utf-8"?>
<chart id="18">
  <P Name="name">Logic/MATLAB Function</P>
  <eml>
    <P Name="name">generateSine</P>
  </eml>
  <Children>
    <state SSID="1">
      <P Name="labelString">eML_blk_kernel()</P>
      <eml>
        <P Name="isEML">1</P>
        <P Name="script">function y = generateSine(phaseDeg, freq, amp, t)
% comment
y = amp * sin(2*pi*freq*t + deg2rad(phaseDeg));
end</P>
      </eml>
    </state>
    <data SSID="4" name="phaseDeg">
      <P Name="scope">INPUT_DATA</P>
      <props>
        <array><P Name="size">-1</P></array>
        <type>
          <P Name="method">SF_INHERITED_TYPE</P>
          <P Name="primitive">SF_DOUBLE_TYPE</P>
        </type>
        <P Name="complexity">SF_COMPLEX_INHERITED</P>
        <unit><P Name="name">inherit</P></unit>
      </props>
      <P Name="dataType">Inherit: Same as Simulink</P>
    </data>
    <data SSID="5" name="y">
      <P Name="scope">OUTPUT_DATA</P>
      <props>
        <array><P Name="size">-1</P></array>
        <type>
          <P Name="method">SF_INHERITED_TYPE</P>
          <P Name="primitive">SF_DOUBLE_TYPE</P>
        </type>
        <P Name="complexity">SF_COMPLEX_INHERITED</P>
        <unit><P Name="name">inherit</P></unit>
      </props>
      <P Name="dataType">Inherit: Same as Simulink</P>
    </data>
  </Children>
</chart>
"#;

    // No machine.xml needed anymore; mapping is derived from chart_*.xml itself

    let base = Utf8PathBuf::from("/simulink/systems");
    let mut files = HashMap::new();
    files.insert(
        base.join("system_root.xml").as_str().to_string(),
        sys_root.to_string(),
    );
    files.insert(
        "/simulink/stateflow/chart_18.xml".to_string(),
        chart_18.to_string(),
    );
    // no machine.xml

    let source = MemSource { files };
    let mut parser = SimulinkParser::new("/", source);

    let system = parser
        .parse_system_file(base.join("system_root.xml"))
        .expect("parse system");
    assert_eq!(system.blocks.len(), 1);
    let blk = &system.blocks[0];
    assert!(
        blk.is_matlab_function,
        "Expected MATLAB Function block flagged"
    );
    // Charts are now pre-parsed and available via parser getters (from chart_*.xml)
    let charts = parser.get_charts();
    let chart = charts.get(&18).expect("chart 18 parsed");
    assert_eq!(chart.id, Some(18));
    assert_eq!(chart.eml_name.as_deref(), Some("generateSine"));
    assert!(
        chart
            .script
            .as_ref()
            .map(|s| s.contains("generateSine"))
            .unwrap_or(false)
    );
    assert!(chart.inputs.iter().any(|p| p.name == "phaseDeg"));
    assert!(chart.outputs.iter().any(|p| p.name == "y"));
    // Also ensure name-based map contains the chart name
    let name_map = parser.get_system_to_chart_map();
    assert_eq!(name_map.get("Logic/MATLAB Function"), Some(&18u32));
}

#[test]
fn matlab_function_blocks_are_captioned_with_their_function_name() {
    use rustylink::model::SlxArchive;

    let file = std::fs::File::open("simulink_test_models/Simulink_Blocks.slx")
        .expect("open Simulink_Blocks.slx");
    let archive = SlxArchive::from_reader(std::io::BufReader::new(file)).expect("read archive");
    let mut system = archive.assembled_root_system().expect("assemble root");
    let (charts, chart_map) = archive.parse_charts();
    rustylink::parser::annotate_matlab_function_names(&mut system, &charts, &chart_map);

    let name_of = |block_name: &str| {
        system
            .blocks
            .iter()
            .find(|b| b.name == block_name)
            .and_then(|b| b.properties.get("MATLABFunctionName"))
            .cloned()
    };
    // Simulink captions a MATLAB Function block with the function its code
    // defines, not with the block's own name.
    assert_eq!(name_of("MATLAB Function").as_deref(), Some("fcn"));
    assert_eq!(name_of("MATLAB Function1").as_deref(), Some("test"));
}

#[test]
fn single_line_function_header() {
    assert_eq!(
        rustylink::parser::chart::script_function_name("function y = fcn(u)\ny = u;"),
        Some("fcn".to_string())
    );
}

#[test]
fn multi_output_function_header() {
    assert_eq!(
        rustylink::parser::chart::script_function_name("function [x,y] = test(u,v)\ny = u;\nx=v;"),
        Some("test".to_string())
    );
}

#[test]
fn function_header_with_continuation_line() {
    let script = "function [out] = ...\n    myFunc(u)\ny = u;";
    assert_eq!(
        rustylink::parser::chart::script_function_name(script),
        Some("myFunc".to_string())
    );
}

#[test]
fn function_header_with_multiple_continuation_lines() {
    let script = "function ...\n  result ...\n  = ...\n  compute(x)\ny = x;";
    assert_eq!(
        rustylink::parser::chart::script_function_name(script),
        Some("compute".to_string())
    );
}

#[test]
fn function_header_no_outputs_with_continuation() {
    let script = "function ...\n  doit(u)\ny = u;";
    assert_eq!(
        rustylink::parser::chart::script_function_name(script),
        Some("doit".to_string())
    );
}

#[test]
fn function_header_after_comment_lines() {
    let script = "% Copyright\n%% cell marker\n\nfunction y = later(u)\ny = u;";
    assert_eq!(
        rustylink::parser::chart::script_function_name(script),
        Some("later".to_string())
    );
}

#[test]
fn commented_out_header_is_ignored() {
    let script = "% function y = wrong(u)\nfunction y = right(u)\ny = u;";
    assert_eq!(
        rustylink::parser::chart::script_function_name(script),
        Some("right".to_string())
    );
}

#[test]
fn header_without_outputs() {
    assert_eq!(
        rustylink::parser::chart::script_function_name("function noOut(u)\ndisp(u);"),
        Some("noOut".to_string())
    );
}

#[test]
fn identifier_starting_with_function_is_not_a_header() {
    assert_eq!(
        rustylink::parser::chart::script_function_name("functions = 3;\ny = functions;"),
        None
    );
}

#[test]
fn no_function_header_returns_none() {
    assert_eq!(
        rustylink::parser::chart::script_function_name("y = u;"),
        None
    );
}

#[test]
fn function_header_with_continuation_in_arguments() {
    let script = concat!(
        "function [q_des, dq_des, ddq_des, running_1__ready_0, q_ref_out] = ...\n",
        "    JojoJointInterpolator(dq_ref, ddq_ref, eigenvalues, dt, ...\n",
        "    consider_position_constraints, consider_velocity_constraints, ...\n",
        "    q_lowerlimit, q_upperlimit, dq_lowerlimit, dq_upperlimit, follow_q_meas, ...\n",
        "    q_target, update_q_target, shortcut_update, reset, dq_measured, q_measured)\n",
        "y = u;"
    );
    assert_eq!(
        rustylink::parser::chart::script_function_name(script),
        Some("JojoJointInterpolator".to_string())
    );
}

#[test]
fn function_header_with_long_name_on_continuation_line() {
    let script = concat!(
        "function [follow_q_meas, control_mode] = ...\n",
        "    Control_Mode_Preprocessor(desired_control_mode, collision_1OK_0_danger, new_collision_trigger, internal_simulation_enabled, is_real_experiment, Robot_DEF) ...\n",
        "%% body\n",
        "follow_q_meas = 0;"
    );
    assert_eq!(
        rustylink::parser::chart::script_function_name(script),
        Some("Control_Mode_Preprocessor".to_string())
    );
}
