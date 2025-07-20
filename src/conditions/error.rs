use derive_more::{Display, Error, From};

/// Error when parsing rule condition expression
#[derive(Debug, Display, From, Error, PartialEq)]
pub enum CondError {
    #[display("Invalid string pattern expression")]
    InvalidPattern,

    #[display("Invalid comparison expression")]
    InvalidComparison,

    #[display("Invalid filetest expression")]
    InvalidFileTest,

    #[display("Quotation never closed in expression")]
    UnclosedQuotation,

    #[display("Rule condition expression is empty")]
    EmptyExpression,

    #[display("Rule conditiion is missing comparison")]
    MissingComparison,

    #[display("Invalid expression suffix")]
    InvalidSuffix,

    #[display("Missing suffix for comparison")]
    MissingSuffix,

    #[display("Condition flags missing brackets")]
    FlagsMissingBrackets,

    #[display("Condition flags are empty")]
    FlagsEmpty,

    #[display("Invalid condition flag")]
    InvalidFlag,
}
