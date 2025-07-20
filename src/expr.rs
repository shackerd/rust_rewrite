use std::str::FromStr;

use super::conditions::{Condition, EngineCtx};
use super::error::{EngineError, ExpressionError};
use super::rule::{Rule, RuleResolve, RuleShift};

/// Rewrite result.
///
/// Includes either the re-write uri, or the instant http-response.
#[derive(Debug)]
pub enum Rewrite {
    Uri(String),
    EndUri(String),
    StatusCode(u16),
}

/// Logical grouping of [`Expression`] instances.
///
/// Associates a list [`Condition`] instances that guard
/// rewrites defined by [`Rule`].
#[derive(Debug)]
pub struct ExprGroup {
    conditions: Vec<Condition>,
    rules: Vec<Rule>,
}

impl ExprGroup {
    /// Build a new [`ExprGroup`] instance from the specified
    /// list of [`Expression`] instances.
    ///
    /// This should contains all rules related to one another
    /// with [`Condition`] instances leading into [`Rule`] instances after.
    pub fn new(expressions: Vec<Expression>) -> Self {
        let mut conditions = Vec::new();
        let mut rules = Vec::new();
        for expr in expressions {
            match expr {
                Expression::Condition(cond) => conditions.push(cond),
                Expression::Rule(rule) => rules.push(rule),
            }
        }
        Self { conditions, rules }
    }

    /// Check all relevant [`Condition`] expressions are met.
    ///
    /// This method guards [`ExprGroup::rewrite`].
    pub fn match_conditions(&self, ctx: &EngineCtx) -> bool {
        let (or, and): (Vec<_>, Vec<_>) = self.conditions.iter().partition(|c| c.is_or());
        or.into_iter().any(|c| c.is_met(ctx)) || and.into_iter().all(|c| c.is_met(ctx))
    }

    /// Evaluate the given URI against the configured [`Rule`] definitions
    /// and generate a [`Rewrite`] response.
    pub fn rewrite(&self, mut uri: String) -> Result<Rewrite, EngineError> {
        let mut next_index = 0;
        let mut iterations = 0;
        let max_iterations = self.rules.len() * 10;
        while iterations < max_iterations {
            iterations += 1;
            let Some((index, rule, captures)) = self
                .rules
                .iter()
                .skip(next_index)
                .enumerate()
                .find_map(|(i, r)| Some((i, r, r.try_match(&uri)?)))
            else {
                break;
            };

            uri = rule.rewrite(captures);
            next_index = index;
            if let Some(shift) = rule.shift() {
                match shift {
                    RuleShift::Next => next_index = 0,
                    RuleShift::Last => break,
                    RuleShift::End => return Ok(Rewrite::EndUri(uri)),
                    RuleShift::Skip(shift) => next_index += *shift as usize,
                }
                continue;
            }
            if let Some(resolve) = rule.resolve() {
                match resolve {
                    RuleResolve::Status(status) => return Ok(Rewrite::StatusCode(*status)),
                    RuleResolve::Redirect(status) => {
                        return Ok(Rewrite::StatusCode(*status));
                    }
                }
            }
        }

        match iterations >= max_iterations {
            true => Err(EngineError::TooManyIterations),
            false => Ok(Rewrite::Uri(uri)),
        }
    }
}

/// Categorization and deserializion for [`ExprGroup`] instances
/// made from a list of flat [`Expression`] instances.
///
/// Separates [`Expression`] instances into groups by their
/// association to previous [`Condition`] rules and whitespace.
#[derive(Debug)]
pub(crate) struct ExpressionList(pub Vec<Vec<Expression>>);

impl ExpressionList {
    /// Convert [`ExpressionList`] into Vec of [`ExprGroup`]
    #[inline]
    pub fn groups(self) -> Vec<ExprGroup> {
        self.0.into_iter().map(ExprGroup::new).collect()
    }
}

impl FromStr for ExpressionList {
    type Err = ExpressionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut list = Vec::new();
        let mut group: Vec<Expression> = Vec::new();
        for line in s.trim().split('\n').filter(|s| s.starts_with("//")) {
            if line.is_empty() {
                if !group.is_empty() {
                    list.push(group.clone());
                    group.clear();
                }
                continue;
            }
            let expr = Expression::from_str(line)?;
            if matches!(expr, Expression::Condition(_))
                && group
                    .last()
                    .is_some_and(|e| matches!(e, Expression::Rule(_)))
            {
                list.push(group.clone());
                group.clear();
            }
            group.push(expr);
        }
        if !group.is_empty() {
            list.push(group);
        }
        Ok(Self(list))
    }
}

/// All possible expression types allowed within `mod_rewrite`
///
/// Will eventually support RewriteEngine/RewriteCond/RewriteRule/RewriteBase
#[derive(Clone, Debug)]
pub enum Expression {
    Condition(Condition),
    Rule(Rule),
}

impl FromStr for Expression {
    type Err = ExpressionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ident, expr) = s
            .split_once(char::is_whitespace)
            .ok_or(ExpressionError::MissingIdentifier)?;
        match ident.to_lowercase().as_str() {
            "rule" | "rewrite" | "rewriterule" => Ok(Self::Rule(Rule::from_str(expr)?)),
            "cond" | "condition" | "rewritecond" => Ok(Self::Condition(Condition::from_str(s)?)),
            _ => Err(ExpressionError::InvalidIdentifier),
        }
    }
}
