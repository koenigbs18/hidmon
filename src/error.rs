use errno::Errno;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error(transparent)]
    Windows(#[from] windows::core::Error),
    #[error(transparent)]
    Unix(#[from] Errno),
}

pub type Result<T> = std::result::Result<T, Error>;
