use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AppError {
    #[error("A valid folder path is required: {message}")]
    InvalidPath { message: String },
    #[error("The path does not exist: {path}")]
    PathNotFound { path: String },
    #[error("The path is not a directory: {path}")]
    NotDirectory { path: String },
    #[error("The folder cannot be read: {path}")]
    PathNotReadable { path: String },
    #[error("Unable to scan {path}: {message}")]
    ScanFailed { path: String, message: String },
    #[error("Unable to persist local history: {message}")]
    PersistenceFailed { message: String },
    #[error("Invalid rule: {message}")]
    InvalidRule { message: String },
    #[error("Unable to parse rule: {message}")]
    RuleParseFailed { message: String },
}
