use std::str::FromStr;

use regex::{Captures, Regex, RegexBuilder};

use super::error::RuleError;

/// Implements the logic and processing for a single re-write expression
/// defined within `mod_rewrite`.
///
/// It contains a regex pattern to match against a request uri,
/// a rewrite string that expands into the new uri, and additional
/// flags that define how the rule behaves within the rule-engine.
///
/// Supports a subset of [offical](https://httpd.apache.org/docs/current/mod/mod_rewrite.html#rewriterule)
/// mod_rewrite rules.
#[derive(Clone, Debug)]
pub struct Rule {
    pattern: Regex,
    rewrite: String,
    flags: Vec<RuleFlag>,
}

impl Rule {
    /// Try to match the rewrite expression pattern to the specified uri.
    ///
    /// Returns a [`Captures`](regex::Captures) group on successful match.
    /// Pass the result into [`RewriteRule::rewrite`] to update the uri.
    #[inline]
    pub fn try_match<'a>(&self, uri: &'a str) -> Option<Captures<'a>> {
        self.pattern.captures(uri)
    }

    /// Takes the result of [`RewriteRule::try_match`] to rewrite
    /// the uri according to the configured rule expression.
    #[inline]
    pub fn rewrite(&self, captures: Captures<'_>) -> String {
        let mut uri = String::new();
        captures.expand(&self.rewrite, &mut uri);
        uri
    }

    /// Retrieves the associated [`RuleShift`] defined in the
    /// expressions flags if any is present.
    #[inline]
    pub fn shift(&self) -> Option<&RuleShift> {
        self.flags.iter().find_map(|f| match f {
            RuleFlag::Shift(shift) => Some(shift),
            _ => None,
        })
    }

    /// Retrieve the associated [`RuleResolve`] defined in the
    /// expressions flags if any is present.
    #[inline]
    pub fn resolve(&self) -> Option<&RuleResolve> {
        self.flags.iter().find_map(|f| match f {
            RuleFlag::Resolve(resolve) => Some(resolve),
            _ => None,
        })
    }
}

impl FromStr for Rule {
    type Err = RuleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut items = s.split_whitespace().filter(|s| !s.is_empty());
        let pattern = items.next().ok_or(RuleError::MissingPattern)?;
        let rewrite = items.next().ok_or(RuleError::MissingRewrite)?.to_string();
        let flags = match items.next() {
            Some(flags) => RuleFlagList::from_str(flags)?.0,
            None => Vec::new(),
        };
        if items.next().is_some() {
            return Err(RuleError::InvalidSuffix);
        }
        let insense = flags.iter().any(|f| f.insensitive());
        Ok(Self {
            pattern: RegexBuilder::new(pattern)
                .case_insensitive(insense)
                .build()?,
            rewrite,
            flags,
        })
    }
}

struct RuleFlagList(Vec<RuleFlag>);

impl FromStr for RuleFlagList {
    type Err = RuleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with('[') || !s.ends_with(']') {
            return Err(RuleError::FlagsMissingBrackets);
        }
        let flags = s[1..s.len() - 1]
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(RuleFlag::from_str)
            .collect::<Result<Vec<RuleFlag>, _>>()?;
        if flags.is_empty() {
            return Err(RuleError::FlagsEmpty);
        }
        let meta = flags.iter().filter(|f| f.is_shift()).count();
        let response = flags.iter().filter(|f| f.is_resolve()).count();
        if (meta + response) > 1 {
            return Err(RuleError::FlagsMutuallyExclusive);
        }
        Ok(Self(flags))
    }
}

#[inline]
fn parse_int(s: &str, default: u16) -> Result<u16, RuleError> {
    match s.is_empty() {
        true => Ok(default),
        false => Ok(u16::from_str(s)?),
    }
}

#[inline]
fn parse_status(s: &str, default: u16) -> Result<u16, RuleError> {
    let status = parse_int(s, default)?;
    match !(100..600).contains(&status) {
        true => Err(RuleError::InvalidFlagStatus),
        false => Ok(status),
    }
}

/// [`RuleFlag`] subtype declaring shift in rule processing after match
#[derive(Clone, Debug)]
pub enum RuleShift {
    End,
    Last,
    Next,
    Skip(u16),
}

/// [`RuleFlag`] subtype declaring a modification in rewrite behavior
#[derive(Clone, Debug)]
pub enum RuleMod {
    NoCase,
}

/// [`RuleFlag`] subtype declaring a final http-response resolution
#[derive(Clone, Debug)]
pub enum RuleResolve {
    Redirect(u16),
    Status(u16),
}

/// Flag Modifiers to a [`RewriteRule`] expression.
///
/// Supports a subset of [official](https://httpd.apache.org/docs/current/rewrite/flags.html)
/// mod_rewrite flags.
#[derive(Clone, Debug)]
pub enum RuleFlag {
    Shift(RuleShift),
    Mod(RuleMod),
    Resolve(RuleResolve),
}

impl RuleFlag {
    #[inline]
    fn insensitive(&self) -> bool {
        matches!(self, Self::Mod(RuleMod::NoCase))
    }
    #[inline]
    fn is_shift(&self) -> bool {
        matches!(self, Self::Shift(_))
    }
    #[inline]
    fn is_resolve(&self) -> bool {
        matches!(self, Self::Resolve(_))
    }
}

impl FromStr for RuleFlag {
    type Err = RuleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (p, s) = match s.split_once('=') {
            Some((prefix, suffix)) => (prefix, suffix),
            None => (s, ""),
        };
        match p.to_lowercase().as_str() {
            "e" | "end" => Ok(Self::Shift(RuleShift::End)),
            "l" | "last" => Ok(Self::Shift(RuleShift::Last)),
            "n" | "next" => Ok(Self::Shift(RuleShift::Next)),
            "s" | "skip" => Ok(Self::Shift(RuleShift::Skip(parse_int(s, 1)?))),
            "i" | "insensitive" | "nc" | "nocase" => Ok(Self::Mod(RuleMod::NoCase)),
            "r" | "redirect" => Ok(Self::Resolve(RuleResolve::Redirect(parse_status(s, 302)?))),
            "f" | "forbidden" => Ok(Self::Resolve(RuleResolve::Status(403))),
            "g" | "gone" => Ok(Self::Resolve(RuleResolve::Status(410))),
            "" => Ok(Self::Resolve(RuleResolve::Status(parse_status(s, 403)?))),
            _ => Err(RuleError::InvalidFlag),
        }
    }
}
