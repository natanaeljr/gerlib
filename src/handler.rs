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

    pub fn get(&mut self, url: &str) -> Result<Response> {
        self.http.headers(&[/*Header::AcceptAppJson*/])?;
        let (code, message) = self.http.get(url)?;
        Ok(Response {
            code: StatusCode::from_u16(code as u16).unwrap(),
            message: message.into(),
        })
    }

    pub fn put(&mut self, url: &str) -> Result<Response> {
        let (code, message) = self.http.post(url, None)?;
        Ok(Response {
            code: StatusCode::from_u16(code as u16).unwrap(),
            message: message.into(),
        })
    }

    pub fn put_json<T>(&mut self, url: &str, data: &T) -> Result<Response>
    where
        T: Serialize + ?Sized,
    {
        self.http
            .headers(&[Header::ContentTypeAppJson /*, Header::AcceptAppJson*/])?;
        let data = serde_json::to_string(data)?;
        let (code, message) = self.http.put(url, Some(data.as_bytes()))?;
        Ok(Response {
            code: StatusCode::from_u16(code as u16).unwrap(),
            message: message.into(),
        })
    }

    pub fn post_json<T>(&mut self, url: &str, data: &T) -> Result<Response>
    where
        T: Serialize + ?Sized,
    {
        self.http
            .headers(&[Header::ContentTypeAppJson /*, Header::AcceptAppJson*/])?;
        let data = serde_json::to_string(data)?;
        let (code, message) = self.http.post(url, Some(data.as_bytes()))?;
        Ok(Response {
            code: StatusCode::from_u16(code as u16).unwrap(),
            message: message.into(),
        })
    }

    pub fn post(&mut self, url: &str) -> Result<Response> {
        let (code, message) = self.http.post(url, None)?;
        Ok(Response {
            code: StatusCode::from_u16(code as u16).unwrap(),
            message: message.into(),
        })
    }

    pub fn delete(&mut self, url: &str) -> Result<Response> {
        self.http.headers(&[/*Header::AcceptAppJson*/])?;
        let (code, message) = self.http.delete(url)?;
        Ok(Response {
            code: StatusCode::from_u16(code as u16).unwrap(),
            message: message.into(),
        })
    }

    pub fn http(self) -> HttpRequestHandler {
        self.http
    }
}

pub struct Response {
    pub code: http::StatusCode,
    pub message: Message,
}

impl Response {
    pub fn expect(self, expected_code: http::StatusCode) -> Result<Message> {
        Ok(self.expect_or(expected_code)?.message)
    }

    pub fn expect_or(self, expected_code: http::StatusCode) -> Result<Self> {
        if self.code.as_u16() != expected_code.as_u16() {
            Err(Error::UnexpectedHttpResponse(self.code, self.message.raw()))
        } else {
            Ok(self)
        }
    }
}

pub struct Message(Vec<u8>);

impl Message {
    pub fn raw(self) -> Vec<u8> {
        self.0
    }

    pub fn string(self) -> String {
        String::from_utf8_lossy(self.0.as_slice()).into()
    }

    pub fn json(self) -> Result<String> {
        const MAGIC_PREFIX: &'static [u8] = b")]}'\n";
        if !self.0.as_slice().starts_with(MAGIC_PREFIX) {
            return Err(Error::NotJsonResponse(self.raw()));
        }
        let json = String::from_utf8_lossy(&self.0[MAGIC_PREFIX.len()..]).into_owned();
        Ok(json)
    }
}

impl From<Vec<u8>> for Message {
    fn from(s: Vec<u8>) -> Self {
        Self(s)
    }
}
