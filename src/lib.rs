#![allow(dead_code)]

extern crate strum;
#[macro_use]
extern crate strum_macros;

use crate::handler::RestHandler;
use crate::http::HttpRequestHandler;
use url::Url;

pub mod accounts;
pub mod changes;
pub mod details;
pub mod error;
pub mod projects;

mod handler;
mod http;
mod r#impl;

pub use crate::http::AuthMethod as HttpAuthMethod;

pub type Result<T> = std::result::Result<T, crate::error::Error>;

/// Gerrit REST API over HTTP.
///
/// The API is suitable for automated tools to build upon, as well as supporting some ad-hoc scripting use cases.
pub struct GerritRestApi {
    rest: RestHandler,
}

impl GerritRestApi {
    /// Create a new GerritRestApi with the host url, username and HTTP password.
    ///
    /// Additional configuration is available through specific methods below.
    pub fn new(base_url: Url, username: &str, password: &str) -> Result<Self> {
        let http = HttpRequestHandler::new(base_url, username, password)?;
        let rest = RestHandler::new(http);
        Ok(Self { rest })
    }

    /// Specify the HTTP authentication method.
    pub fn http_auth(mut self, auth: &HttpAuthMethod) -> Result<Self> {
        self.rest = RestHandler::new(self.rest.http().http_auth(auth)?);
        Ok(self)
    }

    /// Enable/Disable SSL verification of both host and peer.
    pub fn ssl_verify(mut self, enable: bool) -> Result<Self> {
        self.rest = RestHandler::new(self.rest.http().ssl_verify(enable)?);
        Ok(self)
    }
}
