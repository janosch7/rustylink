use serde::{Deserialize, Serialize};

pub const DEFAULT_LIVE_FLOAT_DECIMALS: usize = 2;
pub const LIVE_SCIENTIFIC_LOWER_BOUND: f64 = 1e-1;
pub const LIVE_SCIENTIFIC_UPPER_BOUND: f64 = 1e4;

fn default_live_float_decimals() -> usize {
    DEFAULT_LIVE_FLOAT_DECIMALS
}

fn default_live_scientific_lower_bound() -> f64 {
    LIVE_SCIENTIFIC_LOWER_BOUND
}

fn default_live_scientific_upper_bound() -> f64 {
    LIVE_SCIENTIFIC_UPPER_BOUND
}

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LiveValueDisplayOptions {
    #[serde(default = "default_live_float_decimals")]
    pub float_decimals: usize,
    #[serde(default = "default_live_scientific_lower_bound")]
    pub scientific_lower_bound: f64,
    #[serde(default = "default_live_scientific_upper_bound")]
    pub scientific_upper_bound: f64,
    #[serde(default, alias = "use_scientific")]
    pub always_scientific: bool,
}

impl Default for LiveValueDisplayOptions {
    fn default() -> Self {
        Self {
            float_decimals: DEFAULT_LIVE_FLOAT_DECIMALS,
            scientific_lower_bound: LIVE_SCIENTIFIC_LOWER_BOUND,
            scientific_upper_bound: LIVE_SCIENTIFIC_UPPER_BOUND,
            always_scientific: false,
        }
    }
}

impl LiveValueDisplayOptions {
    pub fn normalized(&self) -> Self {
        let mut normalized = self.clone();
        if normalized.float_decimals == 0 {
            normalized.float_decimals = DEFAULT_LIVE_FLOAT_DECIMALS;
        }
        if !normalized.scientific_lower_bound.is_finite()
            || normalized.scientific_lower_bound <= 0.0
        {
            normalized.scientific_lower_bound = LIVE_SCIENTIFIC_LOWER_BOUND;
        }
        if !normalized.scientific_upper_bound.is_finite()
            || normalized.scientific_upper_bound <= 0.0
        {
            normalized.scientific_upper_bound = LIVE_SCIENTIFIC_UPPER_BOUND;
        }
        if normalized.scientific_lower_bound > normalized.scientific_upper_bound {
            std::mem::swap(
                &mut normalized.scientific_lower_bound,
                &mut normalized.scientific_upper_bound,
            );
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
            display: LiveValueDisplayOptions::default(),
        }
    }

    pub fn with_display(mut self, display: LiveValueDisplayOptions) -> Self {
        self.display = display;
        self
    }

    pub fn first_f64(&self) -> Option<f64> {
        self.value.first_f64()
    }

    pub fn f64_at(&self, index: usize) -> Option<f64> {
        self.value.f64_at(index)
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

    pub fn f64_at(&self, index: usize) -> Option<f64> {
        self.data.f64_at(index)
    }

    pub fn format(&self, display: &LiveValueDisplayOptions) -> String {
        let display = display.normalized();
        let values = self.data.as_string_vec(&display);
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
    Enum(Vec<u64>),
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
            LiveValueList::Enum(values) => values.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
            | LiveValueList::String(_)
            | LiveValueList::Enum(_) => None,
        }
    }

    pub fn f64_at(&self, index: usize) -> Option<f64> {
        match self {
            LiveValueList::Empty => None,
            LiveValueList::Bool(values) => values
                .get(index)
                .map(|value| if *value { 1.0 } else { 0.0 }),
            LiveValueList::Int8(values) => values.get(index).map(|value| *value as f64),
            LiveValueList::Int16(values) => values.get(index).map(|value| *value as f64),
            LiveValueList::Int32(values) => values.get(index).map(|value| *value as f64),
            LiveValueList::Int64(values) => values.get(index).map(|value| *value as f64),
            LiveValueList::UInt8(values) => values.get(index).map(|value| *value as f64),
            LiveValueList::UInt16(values) => values.get(index).map(|value| *value as f64),
            LiveValueList::UInt32(values) => values.get(index).map(|value| *value as f64),
            LiveValueList::UInt64(values) => values.get(index).map(|value| *value as f64),
            LiveValueList::Float32(values) => values.get(index).map(|value| *value as f64),
            LiveValueList::Float64(values) => values.get(index).copied(),
            LiveValueList::Complex32(_)
            | LiveValueList::Complex64(_)
            | LiveValueList::Bytes(_)
            | LiveValueList::String(_)
            | LiveValueList::Enum(_) => None,
        }
    }

    fn as_string_vec(&self, display: &LiveValueDisplayOptions) -> Vec<String> {
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
                .map(|value| format_float(*value as f64, display))
                .collect(),
            LiveValueList::Float64(values) => values
                .iter()
                .map(|value| format_float(*value, display))
                .collect(),
            LiveValueList::Complex32(values) => values
                .iter()
                .map(|value| format_complex(value.re as f64, value.im as f64, display))
                .collect(),
            LiveValueList::Complex64(values) => values
                .iter()
                .map(|value| format_complex(value.re, value.im, display))
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
            LiveValueList::Enum(values) => values.iter().map(ToString::to_string).collect(),
        }
    }
}

pub fn scalar_requires_scientific(value: f64) -> bool {
    scalar_requires_scientific_with_bounds(
        value,
        LIVE_SCIENTIFIC_LOWER_BOUND,
        LIVE_SCIENTIFIC_UPPER_BOUND,
    )
}

pub fn scalar_requires_scientific_with_bounds(
    value: f64,
    lower_bound: f64,
    upper_bound: f64,
) -> bool {
    let abs = value.abs();
    abs != 0.0 && (abs < lower_bound || abs > upper_bound)
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

fn format_float(value: f64, display: &LiveValueDisplayOptions) -> String {
    let decimals = display.float_decimals;
    if display.always_scientific
        || scalar_requires_scientific_with_bounds(
            value,
            display.scientific_lower_bound,
            display.scientific_upper_bound,
        )
    {
        format!("{value:.decimals$e}")
    } else {
        format!("{value:.decimals$}")
    }
}

fn format_complex(re: f64, im: f64, display: &LiveValueDisplayOptions) -> String {
    let re = format_float(re, display);
    let imag_value = if im < 0.0 {
        format_float(-im, display)
    } else {
        format_float(im, display)
    };
    let sign = if im < 0.0 { '-' } else { '+' };
    format!("{re}{sign}{imag_value}i")
}
