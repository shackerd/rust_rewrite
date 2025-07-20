//! Framework agnostic reimplementation of HTTPD's [mod_rewrite](https://httpd.apache.org/docs/current/mod/mod_rewrite.html).
//!
//! # Example
//!
//! ```
//! use mod_rewrite::Engine;
//!
//! let mut engine = Engine::default();
//! engine.add_rules(r#"
//!   RewriteRule /file/(.*)     /tmp/$1      [L]
//!   RewriteRule /redirect/(.*) /location/$1 [R=302]
//!   RewriteRule /blocked/(.*)  -            [F]
//! "#).expect("failed to process rules");
//!
//! let uri = "http://localhost/file/my/document.txt".to_owned();
//! let result = engine.rewrite(uri).unwrap();
//! println!("{result:?}");
//! ```
use std::str::FromStr;

mod conditions;
pub mod error;
mod expr;
mod extra;
mod rule;

use conditions::EngineCtx;
use error::{EngineError, ExpressionError};
use expr::ExpressionList;

pub use conditions::{Condition, context};
pub use expr::{ExprGroup, Expression, Rewrite};
pub use extra::State;
pub use rule::Rule;

/// Expression Engine for Proccessing Rewrite Rules
///
/// Supports a subset of [official](https://httpd.apache.org/docs/current/mod/mod_rewrite.html)
/// `mod_rewrite` expressions.
///
/// # Example
///
/// ```
/// use mod_rewrite::Engine;
///
/// let mut engine = Engine::default();
/// engine.add_rules(r#"
///     RewriteRule /file/(.*)     /tmp/$1      [L]
///     RewriteRule /redirect/(.*) /location/$1 [R=302]
///     RewriteRule /blocked/(.*)  -            [F]
/// "#).expect("failed to process rules");
///
/// let uri = "http://localhost/file/my/document.txt".to_owned();
/// let result = engine.rewrite(uri).unwrap();
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
    ///
    /// This method skips using [`EngineCtx`] which is used to suppliment
    /// [`Condition`] expressions. If you are NOT making use of `RewriteCond`
    /// rules, this method may be simpler to use.
    ///
    /// See [`Engine::rewrite_ctx`] for more details.
    #[inline]
    pub fn rewrite(&self, uri: String) -> Result<Rewrite, EngineError> {
        let ctx = EngineCtx::default();
        self.rewrite_ctx(uri, &ctx)
    }

    /// Evaluate the given URI against the configured [`ExprGroup`] instances
    /// defined and generate a [`Rewrite`] response.
    ///
    /// This method uses an additional [`EngineCtx`] which is used to suppliment
    /// variables expanded in [`Condition`] expressions.
    ///
    /// If your engine is using `RewriteCond` rules, you will want to use this
    /// method with a complete `EngineCtx`. See [`Engine::rewrite`] for a simpler
    /// alternative.
    pub fn rewrite_ctx(&self, mut uri: String, ctx: &EngineCtx) -> Result<Rewrite, EngineError> {
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
