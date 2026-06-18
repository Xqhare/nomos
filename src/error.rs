use nemesis::NemesisError;
use std::fmt;

pub type NomosResult<T> = Result<T, NemesisError>;

#[derive(Debug)]
pub enum NomosError {
    Generic(String),
    Parser(Parser),
    Config(String),
    Io(std::io::Error),
}

#[derive(Debug)]
pub enum Parser {
    Task(String),
    Note(String),
    General(String),
}

impl fmt::Display for Parser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Parser::Task(msg) => write!(f, "{msg}"),
            Parser::Note(msg) => write!(f, "{msg}"),
            Parser::General(msg) => write!(f, "{msg}"),
        }
    }
}

impl fmt::Display for NomosError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NomosError::Config(msg) => write!(f, "{msg}"),
            NomosError::Parser(msg) => write!(f, "{msg}"),
            NomosError::Generic(msg) => write!(f, "{msg}"),
            NomosError::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for NomosError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NomosError::Generic(_) | NomosError::Parser(_) | NomosError::Config(_) => None,
            NomosError::Io(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for NomosError {
    fn from(err: std::io::Error) -> Self {
        NomosError::Io(err)
    }
}
