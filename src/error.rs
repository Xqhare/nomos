use std::fmt;

pub type nomosResult<T> = Result<T, nomosError>;

#[derive(Debug)]
pub enum nomosError {
    Generic(String),
    Io(std::io::Error),
}

impl fmt::Display for nomosError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            nomosError::Generic(msg) => write!(f, "{}", msg),
            nomosError::Io(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for nomosError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            nomosError::Generic(_) => None,
            nomosError::Io(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for nomosError {
    fn from(err: std::io::Error) -> Self {
        nomosError::Io(err)
    }
}
