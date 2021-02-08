use reqwest::header::{AsHeaderName, HeaderValue};
use reqwest::Response;

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Url(url::ParseError),
    InvalidHeaderValue,
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::Reqwest(value)
    }
}

impl From<url::ParseError> for Error {
    fn from(value: url::ParseError) -> Self {
        Error::Url(value)
    }
}

impl Error {
    pub fn check_header(
        response: Response,
        key: impl AsHeaderName,
        val: &'static str,
    ) -> Result<Response, Self> {
        if response.headers().get(key) != Some(&HeaderValue::from_static(val)) {
            return Err(Error::InvalidHeaderValue);
        }

        Ok(response)
    }
}
