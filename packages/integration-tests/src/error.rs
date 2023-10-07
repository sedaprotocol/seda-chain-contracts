use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TestingError {
    #[error("Error executing contract call")]
    ExecuteError(String),
}
