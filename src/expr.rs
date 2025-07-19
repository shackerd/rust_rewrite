use std::str::FromStr;

use super::error::ExpressionError;
use super::rule::RewriteRule;

/// A List of `mod_rewrite` Expressions
#[derive(Debug)]
pub struct ExpressionList(pub Vec<Expression>);

impl FromStr for ExpressionList {
    type Err = ExpressionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.trim()
                .split('\n')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(Expression::from_str)
                .collect::<Result<Vec<Expression>, _>>()?,
        ))
    }
}

/// All possible expression types allowed within `mod_rewrite`
///
/// Will eventually support RewriteEngine/RewriteCond/RewriteRule/RewriteBase
#[derive(Debug)]
pub enum Expression {
    Rule(RewriteRule),
}

impl FromStr for Expression {
    type Err = ExpressionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ident, expr) = s
            .split_once(char::is_whitespace)
            .ok_or(ExpressionError::MissingIdentifier)?;
        match ident.to_lowercase().as_str() {
            "rule" | "rewrite" | "rewriterule" => Ok(Self::Rule(RewriteRule::from_str(expr)?)),
            _ => Err(ExpressionError::InvalidIdentifier),
        }
    }
}
