use nemesis::NemesisError;
use std::fmt;

pub type NomosResult<T> = Result<T, NemesisError>;

#[derive(Debug)]
pub enum NomosError {
    Generic(String),
    Io(std::io::Error),
}

impl fmt::Display for NomosError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NomosError::Generic(msg) => write!(f, "{msg}"),
            NomosError::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for NomosError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NomosError::Generic(_) => None,
            NomosError::Io(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for NomosError {
    fn from(err: std::io::Error) -> Self {
        NomosError::Io(err)
    }
}
