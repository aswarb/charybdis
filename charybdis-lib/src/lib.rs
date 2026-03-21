use std::collections::{BTreeMap, BTreeSet};
const MAX_F64_PRECISION: u8 = 15;
// -------------------------- Config Types

pub trait Validatable<T> {
    fn validate(&self, value: &T) -> Result<(), String>;
}

pub trait ApplyPolicy<T> {
    fn apply_policy(&self, value: T) -> Option<T>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum NanPolicy {
    Allow,
    Reject,
    Default(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShortenNumberPolicy {
    Truncate,
    Round,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShortenTextPolicy {
    EllipsisStart,
    EllipsisEnd,
    CutoffEnd,
    CutoffStart,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SizeConfig {
    pub min: usize,
    pub max: usize,
    pub default: Option<usize>,
}

impl SizeConfig {
    pub fn new(min: usize, max: usize, default: Option<usize>) -> Result<Self, String> {
        if let Some(d) = default {
            if d < min {
                return Err("default size is lower than minimum size".to_string());
            }
            if d > max {
                return Err("default size is larger than maximum size".to_string());
            }
        }
        if max < min {
            return Err("minimum size is larger than maximum size".to_string());
        }
        let config = Self { min, max, default };

        return Ok(config);
    }
}

impl Validatable<usize> for SizeConfig {
    fn validate(&self, value: &usize) -> Result<(), String> {
        if *value < self.min {
            return Err(format!("size {} is below minimum {}", value, self.min));
        }
        if *value > self.max {
            return Err(format!("size {} is above maximum {}", value, self.max));
        }
        return Ok(());
    }
}

impl Validatable<Value> for SizeConfig {
    fn validate(&self, value: &Value) -> Result<(), String> {
        let n = match value {
            Value::Size(n) => n,
            _ => return Err("Expected Size Value".to_string()),
        };
        return self.validate(n);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextConfig {
    pub length: SizeConfig,
    pub default: Option<String>,
    pub shorten_policy: ShortenTextPolicy,
}

impl TextConfig {
    pub fn new(
        length: SizeConfig,
        default: Option<String>,
        shorten_policy: ShortenTextPolicy,
    ) -> Result<Self, String> {
        if let Some(ref d) = default {
            if d.len() < length.min {
                return Err("Length of string less than minimum length".to_string());
            }
            if d.len() > length.max {
                return Err("Length of string more than maximum length".to_string());
            }
        }
        let config = Self {
            length,
            default,
            shorten_policy,
        };

        return Ok(config);
    }
}

impl Validatable<String> for TextConfig {
    fn validate(&self, value: &String) -> Result<(), String> {
        return self.length.validate(&value.len());
    }
}

impl Validatable<Value> for TextConfig {
    fn validate(&self, value: &Value) -> Result<(), String> {
        let s = match value {
            Value::Text(s) => s,
            _ => return Err("Expected Text Value".to_string()),
        };
        return self.validate(s);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberConfig {
    pub min: f64,
    pub max: f64,
    pub default: Option<f64>,
    pub nan_policy: NanPolicy,
}

impl NumberConfig {
    pub fn new(
        min: f64,
        max: f64,
        default: Option<f64>,
        nan_policy: NanPolicy,
    ) -> Result<Self, String> {
        if min > max {
            return Err("min cannot be greater than max".to_string());
        }
        if let Some(d) = default
            && (d < min || d > max)
        {
            return Err("default must be within min/max".to_string());
        }

        let config = Self {
            min,
            max,
            default,
            nan_policy,
        };

        return Ok(config);
    }
}

impl Validatable<Value> for NumberConfig {
    fn validate(&self, value: &Value) -> Result<(), String> {
        let n = match value {
            Value::Number(n) => n,
            _ => return Err("Expected Number Value".to_string()),
        };

        if *n < self.min {
            return Err(format!("value {} is below minimum {}", n, self.min));
        }
        if *n > self.max {
            return Err(format!("value {} is above maximum {}", n, self.max));
        }

        return Ok(());
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContinuousRangeConfig {
    pub number: NumberConfig,
    pub precision: u8,
    pub shorten_type: ShortenNumberPolicy,
}

impl ContinuousRangeConfig {
    pub fn new(
        number: NumberConfig,
        precision: u8,
        shorten_type: ShortenNumberPolicy,
    ) -> Result<Self, String> {
        if precision > MAX_F64_PRECISION {
            return Err(
                format!("precision {precision} is greater than {MAX_F64_PRECISION}").to_string(),
            );
        }
        if let Some(d) = number.default
            && !f64_is_precision(d, precision)
        {
            return Err("default value is of precision greater than allowed".to_string());
        }

        let config = Self {
            number,
            precision,
            shorten_type,
        };

        return Ok(config);
    }
}

impl Validatable<Value> for ContinuousRangeConfig {
    fn validate(&self, value: &Value) -> Result<(), String> {
        return self.number.validate(value);
    }
}

impl ApplyPolicy<f64> for ContinuousRangeConfig {
    fn apply_policy(&self, value: f64) -> Option<f64> {
        if value.is_nan() {
            return match &self.number.nan_policy {
                NanPolicy::Allow => Some(value),
                NanPolicy::Reject => None,
                NanPolicy::Default(d) => Some(*d),
            };
        }

        let shaped = match &self.shorten_type {
            ShortenNumberPolicy::Round => roundf64_to_precision(value, self.precision),
            ShortenNumberPolicy::Truncate => truncatef64(value, self.precision),
        };

        return Some(shaped);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiscreteRangeConfig {
    pub number: NumberConfig,
    pub step: f64,
}

impl DiscreteRangeConfig {
    pub fn new(number: NumberConfig, step: f64) -> Result<Self, String> {
        if step <= 0.0 {
            return Err("step must be positive".to_string());
        }
        if let Some(d) = number.default {
            let steps_from_min = (d - number.min) / step;
            let rounded_steps = steps_from_min.round();

            if !f64_approx_eq(steps_from_min, rounded_steps) {
                eprintln!(
                    "warning: default {} is not reachable from min {} with step {}",
                    d, number.min, step
                );
            }
        }
        let config = Self { number, step };

        return Ok(config);
    }
}

impl Validatable<Value> for DiscreteRangeConfig {
    fn validate(&self, value: &Value) -> Result<(), String> {
        return self.number.validate(value);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeriesConfig {
    pub length: SizeConfig,
    pub values: NumberConfig,
}

impl SeriesConfig {
    pub fn new(length: SizeConfig, values: NumberConfig) -> Result<Self, String> {
        let config = Self { length, values };
        return Ok(config);
    }
}

impl Validatable<Value> for SeriesConfig {
    fn validate(&self, value: &Value) -> Result<(), String> {
        return match value {
            Value::Number(_) => self.values.validate(value),
            Value::Series(items) => {
                let length_check = self.length.validate(&items.len());
                if length_check.is_err() {
                    return length_check;
                }
                for item in items {
                    let item_check = self.values.validate(&Value::Number(*item));
                    if item_check.is_err() {
                        return item_check;
                    }
                }
                Ok(())
            }
            _ => Err("Expected Number or Series Value".to_string()),
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListConfig {
    pub length: SizeConfig,
    pub values: TextConfig,
}

impl ListConfig {
    pub fn new(length: SizeConfig, values: TextConfig) -> Result<Self, String> {
        let config = Self { length, values };
        return Ok(config);
    }
}

impl Validatable<Value> for ListConfig {
    fn validate(&self, value: &Value) -> Result<(), String> {
        return match value {
            Value::Text(_) => self.values.validate(value),
            Value::List(items) => {
                let length_check = self.length.validate(&items.len());
                if length_check.is_err() {
                    return length_check;
                }
                for item in items {
                    let item_check = self.values.validate(item);
                    if item_check.is_err() {
                        return item_check;
                    }
                }
                Ok(())
            }
            _ => Err("Expected Text or List Value".to_string()),
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumConfig {
    pub options: BTreeSet<String>,
    pub default: Option<String>,
}

impl EnumConfig {
    pub fn new(options: BTreeSet<String>, default: Option<String>) -> Result<Self, String> {
        if let Some(ref d) = default
            && !options.contains(d)
        {
            return Err("Default value must be contained in available options".to_string());
        }

        let config = Self { options, default };
        return Ok(config);
    }
}

impl Validatable<Value> for EnumConfig {
    fn validate(&self, value: &Value) -> Result<(), String> {
        let t = match value {
            Value::Text(t) => t,
            _ => return Err("Expected Enum Value".to_string()),
        };

        if !self.options.contains(t) {
            let options_str = self
                .options
                .iter()
                .cloned()
                .collect::<Vec<String>>()
                .join(", ");
            return Err(format!("value {} is not one of [{}]", t, options_str));
        }

        return Ok(());
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BooleanConfig {
    pub default: bool,
}

impl BooleanConfig {
    pub fn new(default: bool) -> Self {
        return Self { default };
    }
}

impl Validatable<Value> for BooleanConfig {
    fn validate(&self, value: &Value) -> Result<(), String> {
        let result = match value {
            Value::Boolean(_) => Ok(()),
            _ => Err("Expected Boolean Value".to_string()),
        };
        return result;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Size(SizeConfig),
    Text(TextConfig),
    Number(NumberConfig),
    Boolean(BooleanConfig),
    ContinuousRange(ContinuousRangeConfig),
    DiscreteRange(DiscreteRangeConfig),
    Series(SeriesConfig),
    List(ListConfig),
    Enum(EnumConfig),
}

impl Validatable<Value> for ValueType {
    fn validate(&self, value: &Value) -> Result<(), String> {
        return match self {
            ValueType::Size(config) => config.validate(value),
            ValueType::Text(config) => config.validate(value),
            ValueType::Number(config) => config.validate(value),
            ValueType::Boolean(config) => config.validate(value),
            ValueType::ContinuousRange(config) => config.validate(value),
            ValueType::DiscreteRange(config) => config.validate(value),
            ValueType::Series(config) => config.validate(value),
            ValueType::List(config) => config.validate(value),
            ValueType::Enum(config) => config.validate(value),
        };
    }
}

pub struct Register {
    pub value_type: ValueType,
    pub value: Value,
    pub actions: BTreeMap<String, String>,
}

pub type Registry = BTreeMap<String, Register>;
// -------------------------- Value Types

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Text(String),
    Number(f64),
    Boolean(bool),
    Size(usize),
    Series(Vec<f64>),
    List(Vec<String>),
}

// -------------------------- Utils

fn f64_approx_eq(a: f64, b: f64) -> bool {
    let diff = (a - b).abs();
    let largest = a.abs().max(b.abs());
    return diff <= largest * f64::EPSILON * 2.0;
}

fn f64_is_precision(value: f64, precision: u8) -> bool {
    let factor = 10_f64.powi(precision as i32);
    let rounded = (value * factor).round() / factor;
    let tolerance = 0.5 / factor;
    return (value - rounded).abs() < tolerance;
}

pub fn roundf64_to_precision(value: f64, precision: u8) -> f64 {
    let factor = 10_f64.powi(precision as i32);
    return (value * factor).round() / factor;
}

pub fn truncatef64(value: f64, precision: u8) -> f64 {
    let factor = 10_f64.powi(precision as i32);
    return (value * factor).trunc() / factor;
}
