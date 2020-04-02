use crate::error::Error;
use crate::http::{Header, HttpRequestHandler};
use http::StatusCode;
use serde::Serialize;

type Result<T> = std::result::Result<T, crate::error::Error>;

pub struct RestHandler {
    http: HttpRequestHandler,
}

impl RestHandler {
    pub fn new(http: HttpRequestHandler) -> Self {
        Self { http }
    }

    pub fn get(&mut self, url: &str, expect_code: StatusCode) -> Result<Response> {
        self.http.headers(&[Header::AcceptAppJson])?;
        let (code, response) = self.http.get(url)?;
        Self::expect_response_code(expect_code.as_u16() as u32, code)?;
        Ok(response.into())
    }

    pub fn put_json<T>(&mut self, url: &str, data: &T, expect_code: StatusCode) -> Result<Response>
    where
        T: Serialize + ?Sized,
    {
        self.http
            .headers(&[Header::ContentTypeAppJson, Header::AcceptAppJson])?;
        let data = serde_json::to_string(data)?;
        let (code, response) = self.http.put(url, Some(data.as_bytes()))?;
        Self::expect_response_code(expect_code.as_u16() as u32, code)?;
        Ok(response.into())
    }

    pub fn post_json<T>(&mut self, url: &str, data: &T, expect_code: StatusCode) -> Result<Response>
    where
        T: Serialize + ?Sized,
    {
        self.http
            .headers(&[Header::ContentTypeAppJson, Header::AcceptAppJson])?;
        let data = serde_json::to_string(data)?;
        let (code, response) = self.http.post(url, Some(data.as_bytes()))?;
        Self::expect_response_code(expect_code.as_u16() as u32, code)?;
        Ok(response.into())
    }

    pub fn delete(&mut self, url: &str, expect_code: StatusCode) -> Result<Response> {
        self.http.headers(&[Header::AcceptAppJson])?;
        let (code, response) = self.http.delete(url)?;
        Self::expect_response_code(expect_code.as_u16() as u32, code)?;
        Ok(response.into())
    }

    fn expect_response_code(expected: u32, actual: u32) -> Result<()> {
        if expected != actual {
            Err(Error::UnexpectedHttpResponse(actual))
        } else {
            Ok(())
        }
    }

    pub fn http(self) -> HttpRequestHandler {
        self.http
    }
}

pub struct Response {
    response: String,
}

impl Response {
    pub fn string(self) -> String {
        self.response
    }

    pub fn json(self) -> Result<String> {
        const MAGIC_PREFIX: &'static str = ")]}'\n";
        if !self.response.starts_with(MAGIC_PREFIX) {
            return Err(Error::NotJsonResponse(self.response));
        }
        let json = self.response[MAGIC_PREFIX.len()..].to_string();
        Ok(json)
    }
}

impl From<String> for Response {
    fn from(s: String) -> Self {
        Self { response: s }
    }
}
