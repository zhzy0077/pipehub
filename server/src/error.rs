use actix_http::body::Body;
use actix_http::http::StatusCode;
use actix_http::{Response, ResponseError};
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Initialization(config::ConfigError),
    DataAccess(String),
    Execution(String),
    Io(std::io::Error),
    Dependency(String),
    Unexpected(String),
    User(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::User(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> Response<Body> {
        let mut resp = Response::new(self.status_code());
        resp.extensions_mut().insert(self.to_string());
        resp.set_body(Body::Empty)
    }
}

impl From<config::ConfigError> for Error {
    fn from(e: config::ConfigError) -> Self {
        Error::Initialization(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<r2d2::Error> for Error {
    fn from(e: r2d2::Error) -> Self {
        Error::DataAccess(e.to_string())
    }
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        Error::DataAccess(e.to_string())
    }
}

impl From<actix_http::Error> for Error {
    fn from(e: actix_http::Error) -> Self {
        Error::Execution(format!("{:?}", e))
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Unexpected(format!("{:?}", e))
    }
}

impl<E> From<actix_threadpool::BlockingError<E>> for Error
where
    E: fmt::Debug,
{
    fn from(e: actix_threadpool::BlockingError<E>) -> Self {
        Error::Execution(format!("{:?}", e))
    }
}

impl From<base58::FromBase58Error> for Error {
    fn from(e: base58::FromBase58Error) -> Self {
        Error::Unexpected(format!("{:?}", e))
    }
}

impl From<actix_http::client::SendRequestError> for Error {
    fn from(e: actix_http::client::SendRequestError) -> Self {
        Error::Dependency(format!("{:?}", e))
    }
}

impl std::convert::From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Dependency(format!("{:?}", e))
    }
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        Error::DataAccess(e.to_string())
    }
}
