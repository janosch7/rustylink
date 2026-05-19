use serde::{Deserialize, Serialize};

pub const DEFAULT_LIVE_FLOAT_DECIMALS: usize = 2;
pub const LIVE_SCIENTIFIC_LOWER_BOUND: f64 = 1e-1;
pub const LIVE_SCIENTIFIC_UPPER_BOUND: f64 = 1e4;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LiveComplex32 {
    pub re: f32,
    pub im: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LiveComplex64 {
    pub re: f64,
    pub im: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct LiveValueDisplayOptions {
    pub float_decimals: usize,
    pub use_scientific: bool,
}

impl LiveValueDisplayOptions {
    pub fn normalized(&self) -> Self {
        let mut normalized = self.clone();
        if normalized.float_decimals == 0 {
            normalized.float_decimals = DEFAULT_LIVE_FLOAT_DECIMALS;
        }
        normalized
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LiveValueEntry {
    pub value: LiveValue,
    #[serde(default)]
    pub display: LiveValueDisplayOptions,
}

impl LiveValueEntry {
    pub fn new(value: LiveValue) -> Self {
        Self {
            value,
            display: LiveValueDisplayOptions {
                float_decimals: DEFAULT_LIVE_FLOAT_DECIMALS,
                use_scientific: false,
            },
        }
    }

    pub fn with_display(mut self, display: LiveValueDisplayOptions) -> Self {
        self.display = display;
        self
    }

    pub fn first_f64(&self) -> Option<f64> {
        self.value.first_f64()
    }

    pub fn formatted_text(&self) -> String {
        self.value.format(&self.display)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LiveValue {
    pub dims: Vec<u32>,
    pub data: LiveValueList,
}

impl LiveValue {
    pub fn new(dims: Vec<u32>, data: LiveValueList) -> Self {
        Self { dims, data }
    }

    pub fn first_f64(&self) -> Option<f64> {
        self.data.first_f64()
    }

    pub fn format(&self, display: &LiveValueDisplayOptions) -> String {
        let display = display.normalized();
        let values = self
            .data
            .as_string_vec(display.float_decimals, display.use_scientific);
        let total = values.len().max(1);

        let raw = match self.dims.as_slice() {
            [rows, cols, ..] if *rows > 0 && *cols > 0 => {
                format_grid(&values, *rows as usize, *cols as usize, true)
            }
            [count] if *count > 0 => format_grid(&values, 1, *count as usize, false),
            _ => format_grid(&values, 1, total, false),
        };

        raw.split(';')
            .map(|row| row.split(',').map(str::trim).collect::<Vec<_>>().join(", "))
            .collect::<Vec<_>>()
            .join(";")
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiveValueList {
    Empty,
    Bool(Vec<bool>),
    Int8(Vec<i8>),
    Int16(Vec<i16>),
    Int32(Vec<i32>),
    Int64(Vec<i64>),
    UInt8(Vec<u8>),
    UInt16(Vec<u16>),
    UInt32(Vec<u32>),
    UInt64(Vec<u64>),
    Float32(Vec<f32>),
    Float64(Vec<f64>),
    Complex32(Vec<LiveComplex32>),
    Complex64(Vec<LiveComplex64>),
    Bytes(Vec<Vec<u8>>),
    String(Vec<String>),
}

impl LiveValueList {
    pub fn len(&self) -> usize {
        match self {
            LiveValueList::Empty => 0,
            LiveValueList::Bool(values) => values.len(),
            LiveValueList::Int8(values) => values.len(),
            LiveValueList::Int16(values) => values.len(),
            LiveValueList::Int32(values) => values.len(),
            LiveValueList::Int64(values) => values.len(),
            LiveValueList::UInt8(values) => values.len(),
            LiveValueList::UInt16(values) => values.len(),
            LiveValueList::UInt32(values) => values.len(),
            LiveValueList::UInt64(values) => values.len(),
            LiveValueList::Float32(values) => values.len(),
            LiveValueList::Float64(values) => values.len(),
            LiveValueList::Complex32(values) => values.len(),
            LiveValueList::Complex64(values) => values.len(),
            LiveValueList::Bytes(values) => values.len(),
            LiveValueList::String(values) => values.len(),
        }
    }

    pub fn first_f64(&self) -> Option<f64> {
        match self {
            LiveValueList::Empty => None,
            LiveValueList::Bool(values) => {
                values.first().map(|value| if *value { 1.0 } else { 0.0 })
            }
            LiveValueList::Int8(values) => values.first().map(|value| *value as f64),
            LiveValueList::Int16(values) => values.first().map(|value| *value as f64),
            LiveValueList::Int32(values) => values.first().map(|value| *value as f64),
            LiveValueList::Int64(values) => values.first().map(|value| *value as f64),
            LiveValueList::UInt8(values) => values.first().map(|value| *value as f64),
            LiveValueList::UInt16(values) => values.first().map(|value| *value as f64),
            LiveValueList::UInt32(values) => values.first().map(|value| *value as f64),
            LiveValueList::UInt64(values) => values.first().map(|value| *value as f64),
            LiveValueList::Float32(values) => values.first().map(|value| *value as f64),
            LiveValueList::Float64(values) => values.first().copied(),
            LiveValueList::Complex32(_)
            | LiveValueList::Complex64(_)
            | LiveValueList::Bytes(_)
            | LiveValueList::String(_) => None,
        }
    }

    fn as_string_vec(&self, decimals: usize, force_scientific: bool) -> Vec<String> {
        match self {
            LiveValueList::Empty => Vec::new(),
            LiveValueList::Bool(values) => values.iter().map(|value| value.to_string()).collect(),
            LiveValueList::Int8(values) => values.iter().map(ToString::to_string).collect(),
            LiveValueList::Int16(values) => values.iter().map(ToString::to_string).collect(),
            LiveValueList::Int32(values) => values.iter().map(ToString::to_string).collect(),
            LiveValueList::Int64(values) => values.iter().map(ToString::to_string).collect(),
            LiveValueList::UInt8(values) => values.iter().map(ToString::to_string).collect(),
            LiveValueList::UInt16(values) => values.iter().map(ToString::to_string).collect(),
            LiveValueList::UInt32(values) => values.iter().map(ToString::to_string).collect(),
            LiveValueList::UInt64(values) => values.iter().map(ToString::to_string).collect(),
            LiveValueList::Float32(values) => values
                .iter()
                .map(|value| format_float(*value as f64, decimals, force_scientific))
                .collect(),
            LiveValueList::Float64(values) => values
                .iter()
                .map(|value| format_float(*value, decimals, force_scientific))
                .collect(),
            LiveValueList::Complex32(values) => values
                .iter()
                .map(|value| {
                    format_complex(value.re as f64, value.im as f64, decimals, force_scientific)
                })
                .collect(),
            LiveValueList::Complex64(values) => values
                .iter()
                .map(|value| format_complex(value.re, value.im, decimals, force_scientific))
                .collect(),
            LiveValueList::Bytes(values) => values
                .iter()
                .map(|value| {
                    value
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>()
                })
                .collect(),
            LiveValueList::String(values) => values.clone(),
        }
    }
}

pub fn scalar_requires_scientific(value: f64) -> bool {
    let abs = value.abs();
    abs != 0.0 && (abs < LIVE_SCIENTIFIC_LOWER_BOUND || abs > LIVE_SCIENTIFIC_UPPER_BOUND)
}

fn format_grid(values: &[String], rows: usize, cols: usize, matrix_view: bool) -> String {
    let mut visual_rows = Vec::with_capacity(rows);
    for row in 0..rows {
        let mut row_values = Vec::with_capacity(cols);
        for col in 0..cols {
            let linear = row * cols + col;
            let index = if matrix_view {
                col * rows + row
            } else {
                linear
            };
            if let Some(value) = values.get(index) {
                row_values.push(value.clone());
            }
        }
        visual_rows.push(row_values.join(","));
    }

    if rows > 1 {
        visual_rows.join(";")
    } else {
        visual_rows.first().cloned().unwrap_or_default()
    }
}

fn format_float(value: f64, decimals: usize, force_scientific: bool) -> String {
    if force_scientific || scalar_requires_scientific(value) {
        format!("{value:.decimals$e}")
    } else {
        format!("{value:.decimals$}")
    }
}

fn format_complex(re: f64, im: f64, decimals: usize, force_scientific: bool) -> String {
    let re = format_float(re, decimals, force_scientific);
    let imag_value = if im < 0.0 {
        format_float(-im, decimals, force_scientific)
    } else {
        format_float(im, decimals, force_scientific)
    };
    let sign = if im < 0.0 { '-' } else { '+' };
    format!("{re}{sign}{imag_value}i")
}

#[cfg(test)]
mod tests {
    use super::{
        LiveValue, LiveValueDisplayOptions, LiveValueEntry, LiveValueList,
        scalar_requires_scientific,
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
            use_scientific: false,
        });

        assert_eq!(entry.formatted_text(), "1.2346");
    }
}
