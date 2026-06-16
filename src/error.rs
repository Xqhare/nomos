use std::fmt;

pub type $NAMEResult<T> = Result<T, $NAMEError>;

#[derive(Debug)]
pub enum $NAMEError {
    Generic(String),
    Io(std::io::Error),
}

impl fmt::Display for $NAMEError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            $NAMEError::Generic(msg) => write!(f, "{}", msg),
            $NAMEError::Io(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for $NAMEError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            $NAMEError::Generic(_) => None,
            $NAMEError::Io(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for $NAMEError {
    fn from(err: std::io::Error) -> Self {
        $NAMEError::Io(err)
    }
}
