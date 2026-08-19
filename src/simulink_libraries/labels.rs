//! Shared metadata-dependent label helpers used by block definitions.
//!
//! These functions implement the `MetadataDependent` variants of
//! [`BlockLabelPolicy`](super::types::BlockLabelPolicy) and
//! [`PortLabelPolicy`](super::types::PortLabelPolicy): they derive a label from
//! the block's extracted [`BlockMetadata`].
//!
//! Property defaults (the value shown when the model omits the property) live in
//! each block's `metadata_keys` via
//! [`MetadataKey::with_default`](super::types::MetadataKey::with_default), so
//! these helpers are thin readers and never hard-code fallbacks themselves.

#![cfg(feature = "egui")]

use crate::model::Block;

use super::metadata::BlockMetadata;

/// Block label = the `Gain` property value.
pub fn gain_value(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    nonempty(meta.get("Gain"))
}

/// Block label = the `Value` property value (a `Constant`'s output value).
pub fn constant_value(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    nonempty(meta.get("Value"))
}

/// Block label = the `GotoTag` property value.
pub fn goto_tag(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    nonempty(meta.get("GotoTag"))
}

/// Logical Operator label: reads `Operator` property (AND/OR/NOT/NAND/NOR/XOR).
pub fn logic_operator(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    nonempty(meta.get("Operator")).map(|s| s.to_uppercase())
}

/// Relational Operator label: maps codes to math symbols.
pub fn relational_operator(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    nonempty(meta.get("Operator")).map(|s| {
        match s.as_str() {
            "<=" => "\u{2264}", // ≤
            ">=" => "\u{2265}", // ≥
            "==" => "=",
            "~=" => "\u{2260}", // ≠
            _ => s.as_str(),
        }
        .to_string()
    })
}

/// Math function label: reads `Operator` (exp, log, sqrt, conj, …).
pub fn math_function(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    nonempty(meta.get("Operator"))
}

/// Bias ("Add Constant") label: reads the `Bias` value and shows it as a signed
/// constant (e.g. `+1`, `-2.5`), matching Simulink's icon.
pub fn bias_value(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    let value = nonempty(meta.get("Bias"))?;
    Some(match value.parse::<f64>() {
        Ok(n) if n >= 0.0 => format!("+{value}"),
        _ => value,
    })
}

/// Inport / Outport label: the port number Simulink writes inside the obround.
pub fn port_number(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    Some(nonempty(meta.get("Port")).unwrap_or_else(|| "1".into()))
}

/// Goto Tag Visibility label: the scoped tag in braces, e.g. `{A}`.
pub fn goto_tag_braced(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    Some(format!(
        "{{{}}}",
        nonempty(meta.get("GotoTag")).unwrap_or_else(|| "A".into())
    ))
}

/// Bus Assignment label: `Bus` over the assignment it performs, e.g.
/// `Bus := signal1` for `AssignedSignals = signal1`.
pub fn bus_assignment(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    Some(match nonempty(meta.get("AssignedSignals")) {
        Some(signals) => format!("Bus\n:= {signals}"),
        None => "Bus".into(),
    })
}

/// String Constant label: the literal it outputs, quoted as Simulink shows it.
pub fn string_constant(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    let value = nonempty(meta.get("String")).unwrap_or_else(|| "\"Hello!\"".into());
    Some(if value.starts_with('"') {
        value
    } else {
        format!("\"{value}\"")
    })
}

/// Bit Clear label: Simulink names the bit it clears, e.g. `Clear bit 0`.
pub fn bit_clear(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    Some(format!(
        "Clear bit {}",
        nonempty(meta.get("iBit")).unwrap_or_else(|| "0".into())
    ))
}

/// Bit Set label: the counterpart of [`bit_clear`], e.g. `Set bit 0`.
pub fn bit_set(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    Some(format!(
        "Set bit {}",
        nonempty(meta.get("iBit")).unwrap_or_else(|| "0".into())
    ))
}

/// Property under which the name of a MATLAB Function block's function is
/// recorded while the model is loaded (it lives in the Stateflow chart, out of
/// reach of the renderers).
///
/// Defined in [`super::stubs`] so the core (non-`egui`) parser can write it.
pub use super::stubs::MATLAB_FUNCTION_NAME_PROPERTY;

/// MATLAB Function label: the name of the function the block runs, e.g. `test`.
pub fn matlab_function_name(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    Some(nonempty(meta.get(MATLAB_FUNCTION_NAME_PROPERTY)).unwrap_or_else(|| "fcn".into()))
}

/// Trigonometry function label: reads `Operator` (sin, cos, acos, atan2, …).
pub fn trig_function(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    nonempty(meta.get("Operator"))
}

/// MinMax label: reads `Function` property (min/max).
pub fn minmax_function(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    nonempty(meta.get("Function")).map(|s| s.to_lowercase())
}

/// MinMax Running Resettable label: the running function applied to the input
/// and the held output, e.g. `min(u,y)`.
pub fn minmax_running_function(block: &Block, meta: &BlockMetadata) -> Option<String> {
    let function = minmax_function(block, meta)?;
    Some(format!("{function}(u,y)"))
}

/// Instance label for a `Compare To Constant` block, derived from its
/// `InstanceData` (`relop`/`const`).  Simulink prints the operator verbatim and
/// the constant without a trailing `.0`, e.g. `<= 3`.
pub fn compare_to_constant(block: &Block) -> Option<String> {
    let id = block.instance_data.as_ref()?;
    let relop = id.properties.get("relop")?.trim();
    let const_val = id.properties.get("const")?.trim();
    Some(format!("{relop} {}", trim_trailing_zeros(const_val)))
}

/// Instance label for a `Compare To Zero` block: the operator against `0`.
pub fn compare_to_zero(block: &Block) -> Option<String> {
    let relop = block
        .instance_data
        .as_ref()
        .and_then(|id| id.properties.get("relop"))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("<=");
    Some(format!("{relop} 0"))
}

/// Is Symmetric caption: `Mode` selects plain or skew symmetry.
pub fn is_symmetric_mode(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    Some(matrix_test_caption(meta, "symmetric"))
}

/// Is Hermitian caption: `Mode` selects plain or skew Hermitian symmetry.
pub fn is_hermitian_mode(_block: &Block, meta: &BlockMetadata) -> Option<String> {
    Some(matrix_test_caption(meta, "hermitian"))
}

/// The property Simulink tests, prefixed with `skew` when `Mode` selects the
/// skew variant (`Skew-Symmetric`, `Skew-Hermitian`).
fn matrix_test_caption(meta: &BlockMetadata, property: &str) -> String {
    if meta
        .get("Mode")
        .is_some_and(|m| m.trim().to_lowercase().starts_with("skew"))
    {
        format!("skew\n{property}")
    } else {
        property.to_string()
    }
}

/// Drop a numeric value's redundant fractional part (`3.0` → `3`).
fn trim_trailing_zeros(value: &str) -> String {
    match value.parse::<f64>() {
        Ok(n) if n.fract() == 0.0 && n.abs() < 1e15 => format!("{}", n as i64),
        _ => value.to_string(),
    }
}

/// Trim a metadata value and discard it if empty.
fn nonempty(value: Option<&str>) -> Option<String> {
    value
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
