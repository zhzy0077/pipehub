use actix_http::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use sqlx::migrate::MigrateError;
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Initialization(config::ConfigError),
    Migrate(MigrateError),
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

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code()).json(crate::Response {
            success: self.status_code().is_success(),
            error_message: self.to_string(),
        })
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

impl From<base58::FromBase58Error> for Error {
    fn from(e: base58::FromBase58Error) -> Self {
        Error::Unexpected(format!("{:?}", e))
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

impl From<MigrateError> for Error {
    fn from(e: MigrateError) -> Self {
        Error::Migrate(e)
    }
}
