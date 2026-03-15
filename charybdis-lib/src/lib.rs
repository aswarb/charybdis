#![allow(clippy::needless_return)]

use std::collections::BTreeSet;
const MAX_F64_PRECISION: u8 = 15;
// -------------------------- Config Types
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

#[derive(Debug, Clone, PartialEq)]
pub struct SeriesConfig {
    pub length: SizeConfig,
    pub values: SizeConfig,
}
impl SeriesConfig {
    pub fn new(length: SizeConfig, values: SizeConfig) -> Result<Self, String> {
        let config = Self { length, values };
        return Ok(config);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListConfig {
    pub length: SizeConfig,
    pub values: SizeConfig,
}

impl ListConfig {
    pub fn new(length: SizeConfig, values: SizeConfig) -> Result<Self, String> {
        let config = Self { length, values };
        return Ok(config);
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

#[derive(Debug, Clone, PartialEq)]
pub struct BooleanConfig {
    pub default: bool,
}

impl BooleanConfig {
    pub fn new(default: bool) -> Self {
        return Self { default };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Text(TextConfig),
    Number(NumberConfig),
    Boolean(BooleanConfig),
    ContinuousRange(ContinuousRangeConfig),
    DiscreteRange(DiscreteRangeConfig),
    Series(SeriesConfig),
    List(ListConfig),
    Enum(NumberConfig),
}

// -------------------------- Value Types

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Text(String),
    Number(f64),
    Boolean(bool),
    Range(f64),
    Series(Vec<f64>),
    List(Vec<String>),
    Enum(String),
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
// -------------------------- Default

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
