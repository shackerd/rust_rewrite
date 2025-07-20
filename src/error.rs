use std::num::ParseIntError;

use derive_more::{Display, Error, From};

use super::conditions::CondError;

/// Errors when running expression engine
#[derive(Debug, Display, From, Error)]
pub enum EngineError {
    #[display("Too many iterations on rule processing. Infintite loop")]
    TooManyIterations,
}

/// Errors when parsing all rewrite expressions
#[derive(Debug, Display, From, Error)]
pub enum ExpressionError {
    #[display("Missing expression identifier")]
    MissingIdentifier,

    #[display("Invalid rule identifier")]
    InvalidIdentifier,

    /// Error when parsing condition rule
    ConditionError(CondError),

    /// Error when parsing rewrite rule
    RuleError(RuleError),
}

/// Errors when parsing rewrite rules
#[derive(Debug, Display, From, Error)]
pub enum RuleError {
    #[display("Rule is missing a pattern")]
    MissingPattern,

    #[display("Invalid regex in rule rewrite pattern")]
    InvalidRegex(regex::Error),

    #[display("Rule is missing a rewrite expression")]
    MissingRewrite,

    #[display("Invalid suffix to rule expression")]
    InvalidSuffix,

    #[display("Rule flag definitions missing brackets")]
    FlagsMissingBrackets,

    #[display("Rule flags empty")]
    FlagsEmpty,

    #[display("Rule flags used are mutually exclusive")]
    FlagsMutuallyExclusive,

    #[display("Invalid flag in rule definition")]
    InvalidFlag,

    #[display("Invalid number in rule definition")]
    InvalidFlagNumber(ParseIntError),

    #[display("Invalid status code in rule definition")]
    InvalidFlagStatus,
}
