use std::str::FromStr;

mod error;
mod expr;
mod rule;

pub use error::{EngineError, ExpressionError, RuleError};
pub use expr::{Expression, ExpressionList};
pub use rule::{RewriteRule, RuleFlag, RuleMod, RuleResolve, RuleShift};

/// [`Engine`] official rewrite result.
///
/// Includes either the re-write uri, or the instant http-response.
#[derive(Debug)]
pub enum Rewrite {
    Uri(String),
    StatusCode(u16),
}

/// Expression Engine for Proccessing Rewrite Rules
///
/// Supports a subset of [official](https://httpd.apache.org/docs/current/mod/mod_rewrite.html)
/// mod_rewrite expressions.
///
/// # Example
///
/// ```
/// use mod_rewrite::Engine;
///
/// let engine = Engine::from_rules(r#"
///     Rewrite /file/(.*)     /tmp/$1      [L]
///     Rewrite /redirect/(.*) /location/$1 [R=302]
///     Rewrite /blocked/(.*)  -            [F]
/// "#).expect("failed to process rules");
///
/// let uri = "http://localhost/file/my/document.txt".to_owned();
/// let result = engine.rewrite(uri).unwrap();
/// println!("{result:?}");
/// ```
#[derive(Debug, Default)]
pub struct Engine {
    rules: Vec<RewriteRule>,
}

impl Engine {
    /// Generate a new instance of [`Engine`] from a string containing
    /// `mod_rewrite` expressions.
    #[inline]
    pub fn from_rules(rules: &str) -> Result<Self, ExpressionError> {
        Self::from_str(rules)
    }

    /// Evaluate the given URI against the engines configured rules
    /// and generate a rewrite response.
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

impl FromStr for Engine {
    type Err = ExpressionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut engine = Self::default();
        let expr_list = ExpressionList::from_str(s)?;
        for expr in expr_list.0 {
            match expr {
                Expression::Rule(rule) => engine.rules.push(rule),
            }
        }
        Ok(engine)
    }
}
