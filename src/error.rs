/*

Error handling and the Error type for the project, all errors converted into standarized output

 */
use std::
    fmt::{Debug, Display};
    
pub enum Error {
    MissingEntryInDatabase(String),
    TroubleSavingLoginInfo(String),
    ReqwestError(reqwest::Error),
    PixelaError(String),
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match &self {
            MissingEntryInDatabase(msg) => write!(f, "Missing data in database: {}", msg),
            TroubleSavingLoginInfo(msg) => write!(f, "{:?}", msg),
            ReqwestError(err) => write!(f, "{:?}", err),
            PixelaError(msg) => write!(f, "Api call failed! Pixela responded with: {}",msg)
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match &self {
            MissingEntryInDatabase(err) => write!(f, "{}", err),
            TroubleSavingLoginInfo(err) => write!(f, "There was an error with saving your data: {err}"),
            ReqwestError(err) => write!(f, "{}", err),
            PixelaError(msg) => write!(f, "Api call failed! Pixela responded with: {}",msg)


        }
    }
}
impl From<&str> for Error {
    fn from(string: &str) -> Self {
        Self::MissingEntryInDatabase(string.to_string())
    }
}
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::MissingEntryInDatabase(err.to_string())
    }
}
impl From<sled::Error> for Error {
    fn from(err: sled::Error) -> Self {
        match err {
            sled::Error::Io(io_err) => Error::from(io_err),
            _ => err.into(),
        }
    }
}
pub type Result<T> = std::result::Result<T, Error>;
