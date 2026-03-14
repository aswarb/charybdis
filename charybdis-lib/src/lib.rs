#[derive(Debug, Clone, PartialEq)]
pub enum NanPolicy {
    Allow,
    Reject,
    Default,
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
pub struct TextConfig {
    pub length: NumberConfig,
    pub default: Option<String>,
    pub shorten_policy: ShortenTextPolicy,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberConfig {
    pub min: f64,
    pub max: f64,
    pub default: Option<f64>,
    pub nan_policy: NanPolicy,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContinuousRangeConfig {
    pub number: NumberConfig,
    pub precision: u8,
    pub shorten_type: ShortenNumberPolicy,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiscreteRangeConfig {
    pub number: NumberConfig,
    pub step: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeriesConfig {
    pub length: NumberConfig,
    pub values: NumberConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListConfig {
    pub length: NumberConfig,
    pub values: NumberConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumConfig {
    pub options: Vec<String>,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BooleanConfig {
    pub default: bool,
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
