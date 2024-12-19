#![allow(clippy::enum_variant_names)]
use crate::accelerator::AcceleratorParseError;

#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    NotAChildOfThisMenu,
    NotInitialized,
    AlreadyInitialized,
    AcceleratorParseError(AcceleratorParseError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotAChildOfThisMenu => write!(
                f,
                "This menu item is not a child of this `Menu` or `Submenu`"
            ),
            Error::NotInitialized => write!(f, "This menu has not been initialized for this hwnd"),
            Error::AlreadyInitialized => {
                write!(f, "This menu has already been initialized for this hwnd")
            }
            Error::AcceleratorParseError(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::AcceleratorParseError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<AcceleratorParseError> for Error {
    fn from(err: AcceleratorParseError) -> Self {
        Error::AcceleratorParseError(err)
    }
}

/// Convenient type alias of Result type for muda.
pub type Result<T> = std::result::Result<T, Error>;
