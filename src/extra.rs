use std::str::FromStr;

use super::error::ExpressionError;

/// Singular `RewriteEngine` expression definition.
///
/// Considered a breakpoint for [`ExprGroup`](super::ExprGroup)
/// and enables/disables the entire group based on the configured
/// state.
#[derive(Clone, Debug, Default)]
pub enum State {
    #[default]
    On,
    Off,
}

impl FromStr for State {
    type Err = ExpressionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "on" => Ok(Self::On),
            "off" => Ok(Self::Off),
            _ => Err(ExpressionError::InvalidStateRule),
        }
    }
}
