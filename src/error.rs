use std::env;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use teloxide::RequestError;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Env(env::VarError),
    Sqlx(sqlx::Error),
    Request(RequestError),
    RequestFailed(reqwest::Error),
    ProfitAuthFailed,
    Parse(ParseIntError),
}

// region:    ---From

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::RequestFailed(value)
    }
}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Error::Parse(value)
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Error::Sqlx(value)
    }
}

impl From<env::VarError> for Error {
    fn from(value: env::VarError) -> Self {
        Error::Env(value)
    }
}

impl From<RequestError> for Error {
    fn from(value: RequestError) -> Self {
        Error::Request(value)
    }
}

// endregion: ---From

// region:    --- Error boilerplate
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
// endregion: --- Error boilerplate
