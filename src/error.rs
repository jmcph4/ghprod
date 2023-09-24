use std::{error::Error, fmt};

use ethers::prelude::ProviderError; /* TODO: remove */

pub const ERROR_CLIENT_INIT: u8 = 1u8;

#[derive(Debug)]
pub enum InnerGhProdError {
    ClientError(ProviderError),
}

impl fmt::Display for InnerGhProdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ClientError(e) => write!(f, "ClientError: {}", e),
        }
    }
}

impl Error for InnerGhProdError {}

#[derive(Debug)]
pub struct GhProdError {
    code: u8,
    msg: String,
    inner: Option<InnerGhProdError>,
}

impl GhProdError {
    pub fn new(code: u8, msg: String, inner: Option<InnerGhProdError>) -> Self {
        Self { code, msg, inner }
    }
}

impl fmt::Display for GhProdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.inner {
            Some(e) => write!(f, "E{}: {} due to {}", self.code, self.msg, e),
            None => write!(f, "E{}: {}", self.code, self.msg),
        }
    }
}

impl Error for GhProdError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.inner.as_ref().map(|x| x as &dyn Error)
    }
}

impl From<ProviderError> for GhProdError {
    fn from(value: ProviderError) -> Self {
        Self::new(
            ERROR_CLIENT_INIT,
            "Failed to initialise client".to_string(),
            Some(InnerGhProdError::ClientError(value)),
        )
    }
}
