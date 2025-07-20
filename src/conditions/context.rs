use std::{fmt::Debug, net::SocketAddr};

use once_cell::sync::Lazy;
use regex::Regex;

static MATCHER: Lazy<Regex> = Lazy::new(|| Regex::new(r"%\{\w+\}").unwrap());

macro_rules! get {
    ($key:expr) => {
        Some($key.as_ref().map(|s| s.as_str()).unwrap_or(""))
    };
}

macro_rules! setter {
    ($key:ident) => {
        pub fn $key(mut self, $key: String) -> Self {
            self.$key = Some($key);
            self
        }
    };
}

#[derive(Debug, Default)]
pub struct EngineCtx<'a> {
    date: Option<&'a DateCtx>,
    request: Option<&'a RequestCtx>,
    server: Option<&'a ServerCtx>,
}

impl<'a> EngineCtx<'a> {
    /// Assign [`DateCtx`] sub-context to the complete [`EngineCtx`]
    pub fn date_ctx(mut self, date: Option<&'a DateCtx>) -> Self {
        self.date = date;
        self
    }

    /// Assign [`RequestCtx`] sub-context to the complete [`EngineCtx`]
    pub fn request_ctx(mut self, request: Option<&'a RequestCtx>) -> Self {
        self.request = request;
        self
    }

    /// Assign [`ServerCtx`] sub-context to the complete [`EngineCtx`]
    pub fn server_ctx(mut self, server: Option<&'a ServerCtx>) -> Self {
        self.server = server;
        self
    }

    /// Return the equivalent value associated with the specified
    /// variable expression.
    pub fn fill(&self, expr: &str) -> &str {
        self.date
            .and_then(|ctx| ctx.fill(expr))
            .or(self.request.and_then(|ctx| ctx.fill(expr)))
            .or(self.server.and_then(|ctx| ctx.fill(expr)))
            .unwrap_or("")
    }

    /// Replace all variables within expression with data
    /// specified within with the [`EngineCtx`] and return
    /// the updated string.
    pub fn replace_all(&self, expr: &str) -> String {
        MATCHER
            .find_iter(expr)
            .map(|c| c.as_str().to_owned())
            .fold(expr.to_owned(), |acc, key| {
                let attr = key.trim_matches(|c| ['%', '{', '}'].contains(&c));
                acc.replace(&key, self.fill(attr))
            })
    }
}

#[derive(Debug)]
pub struct DateCtx {
    time_year: String,
    time_month: String,
    time_day: String,
    time_hour: String,
    time_min: String,
    time_sec: String,
    time_wday: String,
    time: String,
}

impl DateCtx {
    pub fn new() -> Self {
        let date = chrono::Local::now();
        Self {
            time_year: date.format("%Y").to_string(),
            time_month: date.format("%m").to_string(),
            time_day: date.format("%d").to_string(),
            time_hour: date.format("%H").to_string(),
            time_min: date.format("%M").to_string(),
            time_sec: date.format("%S").to_string(),
            time_wday: date.format("%A").to_string(),
            time: date.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

impl DateCtx {
    fn fill(&self, key: &str) -> Option<&str> {
        match key {
            "TIME_YEAR" => Some(self.time_year.as_str()),
            "TIME_MONTH" => Some(self.time_month.as_str()),
            "TIME_DAY" => Some(self.time_day.as_str()),
            "TIME_HOUR" => Some(self.time_hour.as_str()),
            "TIME_MIN" => Some(self.time_min.as_str()),
            "TIME_SEC" => Some(self.time_sec.as_str()),
            "TIME_WDAY" => Some(self.time_wday.as_str()),
            "TIME" => Some(self.time.as_str()),
            _ => None,
        }
    }
}

impl Default for DateCtx {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default)]
pub struct ServerCtx {
    document_root: Option<String>,
    server_addr: Option<String>,
    server_admin: Option<String>,
    server_name: Option<String>,
    server_port: Option<String>,
    server_protocol: Option<String>,
    server_software: Option<String>,
}

impl ServerCtx {
    setter!(document_root);
    setter!(server_admin);
    setter!(server_name);
    setter!(server_protocol);
    setter!(server_software);

    pub fn server_addr(mut self, server_addr: impl Into<SocketAddr>) -> Self {
        let addr: SocketAddr = server_addr.into();
        self.server_addr = Some(addr.to_string());
        self.server_name = Some(self.server_name.unwrap_or_else(|| addr.ip().to_string()));
        self.server_port = Some(addr.port().to_string());
        self
    }
}

impl ServerCtx {
    fn fill(&self, key: &str) -> Option<&str> {
        match key {
            "DOCUMENT_ROOT" => get!(self.document_root),
            "SERVER_ADMIN" => get!(self.server_admin),
            "SERVER_ADDR" => get!(self.server_addr),
            "SERVER_NAME" => get!(self.server_name),
            "SERVER_PORT" => get!(self.server_port),
            "SERVER_PROTOCOL" => get!(self.server_protocol),
            "SERVER_SOFTWARE" => get!(self.server_software),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct RequestCtx {
    auth_type: Option<String>,
    ipv6: Option<String>,
    path_info: Option<String>,
    query_string: Option<String>,
    remote_addr: Option<String>,
    remote_host: Option<String>,
    remote_port: Option<String>,
    request_method: Option<String>,
    request_uri: Option<String>,
}

impl RequestCtx {
    setter!(auth_type);
    setter!(path_info);
    setter!(query_string);
    setter!(request_method);
    setter!(request_uri);

    pub fn remote_addr(mut self, remote_addr: impl Into<SocketAddr>) -> Self {
        let addr: SocketAddr = remote_addr.into();
        self.remote_addr = Some(addr.to_string());
        self.remote_host = Some(addr.ip().to_string());
        self.remote_port = Some(addr.port().to_string());
        self
    }
}

impl RequestCtx {
    fn fill(&self, key: &str) -> Option<&str> {
        match key {
            "AUTH_TYPE" => get!(self.auth_type),
            "IPV6" => get!(self.ipv6),
            "PATH_INFO" => get!(self.path_info),
            "QUERY_STRING" => get!(self.query_string),
            "REMOTE_ADDR" => get!(self.remote_addr),
            "REMOTE_HOST" => get!(self.remote_host),
            "REMOTE_PORT" => get!(self.remote_port),
            "REQUEST_METHOD" => get!(self.request_method),
            "REQUEST_URI" => get!(self.request_uri),
            _ => None,
        }
    }
}
