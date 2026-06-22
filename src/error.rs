use nemesis::NemesisError;
use std::fmt;

/// Result type
pub type NomosResult<T> = Result<T, NemesisError>;

/// Error type
#[derive(Debug)]
pub enum NomosError {
    /// Generic error - For development
    Generic(String),
    /// Parser error
    Parser(Parser),
    /// Config error
    Config(String),
    /// Task error
    Task(String),
    /// IO error wrapper for Nemesis compatibility
    Io(std::io::Error),
    /// CLI error
    CLI(String),
}

/// Parser error
#[derive(Debug)]
pub enum Parser {
    /// Parser error during task parsing
    Task(String),
    /// Parser error during note parsing
    Note(String),
    /// General parser error
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
            NomosError::CLI(msg) => write!(f, "{msg}"),
            NomosError::Task(msg) => write!(f, "{msg}"),
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
            NomosError::Generic(_)
            | NomosError::Parser(_)
            | NomosError::CLI(_)
            | NomosError::Task(_)
            | NomosError::Config(_) => None,
            NomosError::Io(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for NomosError {
    fn from(err: std::io::Error) -> Self {
        NomosError::Io(err)
    }
}
