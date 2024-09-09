// -- Contains errors and formating for errors.
use crate::util;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(thiserror::Error, core::fmt::Debug)]
pub enum Error {
    #[error("{0}")]
    Custom(String),

    #[error("Seed not found: {0}")]
    SeedNotFoundError(String),

    #[error("Credentials not found: {0}")]
    CredentialsNotFoundError(String),

    #[error(transparent)]
    UtilError(#[from] util::UtilsError),
}

impl Error {
    /// Sets a custom error,
    pub fn custom(e: impl std::fmt::Display) -> Error {
        Error::Custom(e.to_string())
    }

    pub fn seed_not_found(e: impl std::fmt::Display) -> Error {
        Error::SeedNotFoundError(e.to_string())
    }

    pub fn credentials_not_found(e: impl std::fmt::Display) -> Error {
        Error::CredentialsNotFoundError(e.to_string())
    }
}

// -- Disable hints for now as it would require to get rid of thiserror. Might
// be a nice future improvement.
//impl Error {
//    fn hint(&self) -> Option<&'static str> {
//        match self {
//            Error::Custom(_) => None,
//            Error::SeedNotFoundError(_) => {
//                Some("check if data_dir is correct or register a node first")
//            }
//            Error::CredentialsNotFoundError(_) => {
//                Some("check if data_dir is correct or try to recover")
//            }
//            Error::UtilError(_) => None,
//        }
//    }
//}
//
//impl Debug for Error {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        if let Some(hint) = self.hint() {
//            write!(f, "error: {}\nhint: {}", self.to_string(), hint)
//        } else {
//            write!(f, "{}", self.to_string())
//        }
//    }
//}
