/*

Error handling and the Error type for the project, all errors converted into standarized output

 */
use std::{
     fmt::{write, Debug, Display}, io};
    
pub enum Error {
    MissingEntryInDatabase(String),
    TroubleSavingLoginInfo(String),
    ReqwestError(reqwest::Error),
    PixelaError(String),
    SumGraphError(SumGraphError),
}
#[derive(Debug)]
pub enum SumGraphErrorKind {
    RepeatingNames,
    ErrorIOFile(std::io::Error),
    IncorrectNames,
    GraphNotFoundLocally,
    GraphsSumEachOther,

}

#[derive(Debug)]
pub struct SumGraphError {
    kind: SumGraphErrorKind,
    msg: Option<String>
}
impl SumGraphError {
    pub fn new(kind: SumGraphErrorKind) -> SumGraphError {
        let msg = match kind {
            SumGraphErrorKind::RepeatingNames => None,
            SumGraphErrorKind::ErrorIOFile(ref err) => Some(err.to_string()),
            SumGraphErrorKind::IncorrectNames => Some(String::from("Graphs with such names do not exist for your username.")),
            SumGraphErrorKind::GraphNotFoundLocally => Some(String::from("Couldn't locate a sum graph with provided name in your config.")),
            SumGraphErrorKind::GraphsSumEachOther => Some(String::from("Graphs sum each other which leads to unexpected behaviour, ex: SumGraphA {SumGraphB...}, SumGraphB{SumGraphA...}")),
        };

        SumGraphError { kind, msg }
    }
}
impl Display for SumGraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            SumGraphErrorKind::RepeatingNames => write!(f,"There are repeated names of sum graphs."),
            _ => write!(f, "{}", self.msg.as_ref().unwrap()),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            MissingEntryInDatabase(msg) => write!(f, "Missing data in database: {}", msg),
            TroubleSavingLoginInfo(msg) => write!(f, "{:?}", msg),
            ReqwestError(err) => write!(f, "{:?}", err),
            PixelaError(msg) => write!(f, "Api call failed! Pixela responded with: {}",msg),
            SumGraphError(err) => write!(f, "{:?}", err), 
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
            PixelaError(msg) => write!(f, "Api call failed! Pixela responded with: {}",msg),
            SumGraphError(err) => write!(f, "{}", err), 

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
