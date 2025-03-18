use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use teloxide::RequestError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    // -- Config
    ConfigMissingEnv(&'static str),
    ConfigWrongFormat(&'static str),

    Sqlx(sqlx::Error),
    Request(RequestError),
    RequestFailed(reqwest::Error),
    ProfitAuthFailed,
    ProfitGetDataFailed,
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
