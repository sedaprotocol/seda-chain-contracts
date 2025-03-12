use hex::FromHexError;
use thiserror::Error;
use vrf_rs::error::VrfError;

// We need it to be cloneable for tests :c
#[derive(Error, Debug, PartialEq)]
#[cfg_attr(feature = "test-utils", derive(Clone))]
pub enum Error {
    #[error(transparent)]
    FromHex(#[from] FromHexError),

    #[error(transparent)]
    FromBase64(#[from] base64::DecodeError),

    #[cfg(not(feature = "test-utils"))]
    #[error(transparent)]
    Prove(#[from] VrfError),

    #[cfg(feature = "test-utils")]
    #[error("{0}")]
    Prove(String),
}

#[cfg(feature = "test-utils")]
impl From<VrfError> for Error {
    fn from(err: VrfError) -> Self {
        Error::Prove(err.to_string())
    }
}

pub type Result<T> = core::result::Result<T, Error>;
