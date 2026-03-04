use serde::Serialize;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Sdk(#[from] glsdk::Error),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Bip39(#[from] bip39::Error),

    #[error("{0}")]
    Json(#[from] serde_json::Error),

    #[error("Phrase not found: {0}")]
    PhraseNotFound(String),

    #[error("Credentials not found: {0}")]
    CredentialsNotFound(String),

    #[error("{0}")]
    Other(String),
}

#[derive(Serialize)]
struct ErrorJson {
    error: String,
}

impl Error {
    pub fn print_and_exit(&self) -> ! {
        let json = serde_json::to_string(&ErrorJson {
            error: self.to_string(),
        })
        .unwrap_or_else(|_| format!("{{\"error\":\"{}\"}}", self));
        eprintln!("{}", json);
        std::process::exit(1);
    }
}
