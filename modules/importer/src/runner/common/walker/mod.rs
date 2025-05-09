mod dir;
mod git;

pub use dir::*;
pub use git::*;
use std::path::{Path, PathBuf};
pub enum CallbackError {
    /// Operation should be canceled
    Canceled,
    /// Process error
    Processing(anyhow::Error),
}

pub trait Callbacks<T>: Send + 'static {
    /// Handle an error while loading the file
    #[allow(unused)]
    fn loading_error(&self, path: PathBuf, message: String) {}

    /// Process the file.
    ///
    /// Any error returned will terminate the walk with a critical error.
    #[allow(unused)]
    fn process(&self, path: &Path, document: T) -> Result<(), CallbackError> {
        Ok(())
    }

    fn is_canceled(&self) -> bool {
        false
    }
}

impl<T> Callbacks<T> for () {}
