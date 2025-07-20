use std::str::FromStr;

mod conditions;
mod error;
mod expr;
mod rule;

pub use conditions::*;
pub use error::{EngineError, ExpressionError, RuleError};
pub use expr::{ExprGroup, Expression, ExpressionList, Rewrite};
pub use rule::{Rule, RuleFlag, RuleMod, RuleResolve, RuleShift};

/// Expression Engine for Proccessing Rewrite Rules
///
/// Supports a subset of [official](https://httpd.apache.org/docs/current/mod/mod_rewrite.html)
/// mod_rewrite expressions.
///
/// # Example
///
/// ```
/// use mod_rewrite::{Engine, EngineCtx};
///
/// let mut engine = Engine::default();
/// engine.add_rules(r#"
///     Rewrite /file/(.*)     /tmp/$1      [L]
///     Rewrite /redirect/(.*) /location/$1 [R=302]
///     Rewrite /blocked/(.*)  -            [F]
/// "#).expect("failed to process rules");
///
/// let ctx = EngineCtx::default();
/// let uri = "http://localhost/file/my/document.txt".to_owned();
/// let result = engine.rewrite(uri, &ctx).unwrap();
/// println!("{result:?}");
/// ```
#[derive(Debug, Default)]
pub struct Engine {
    groups: Vec<ExprGroup>,
}

impl Engine {
    /// Parse additonal [`Expression`]s to append as [`ExprGroup`]s to the
    /// existing engine.
    #[inline]
    pub fn add_rules(&mut self, rules: &str) -> Result<(), ExpressionError> {
        let groups = ExpressionList::from_str(rules)?.groups();
        self.groups.extend(groups);
        Ok(())
    }

    /// Evaluate the given URI against the configured [`ExprGroup`] instances
    /// defined and generate a [`Rewrite`] response.
    pub fn rewrite(&self, mut uri: String, ctx: &EngineCtx) -> Result<Rewrite, EngineError> {
        for group in self.groups.iter().filter(|g| g.match_conditions(ctx)) {
            uri = match group.rewrite(uri)? {
                Rewrite::Uri(uri) => uri,
                status => return Ok(status),
            };
        }
        Ok(Rewrite::Uri(uri))
    }
}

impl FromStr for Engine {
    type Err = ExpressionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let groups = ExpressionList::from_str(s)?.groups();
        Ok(Self { groups })
    }
}
