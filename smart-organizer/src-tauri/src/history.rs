use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::errors::AppError;
use crate::filesystem::OperationTransaction;

const HISTORY_FILE: &str = "transactions.json";
const MAX_TRANSACTIONS: usize = 50;

fn history_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| AppError::PersistenceFailed {
            message: format!("Unable to resolve the app data directory: {error}"),
        })?;
    fs::create_dir_all(&directory).map_err(|error| AppError::PersistenceFailed {
        message: format!("Unable to create the app data directory: {error}"),
    })?;
    Ok(directory.join(HISTORY_FILE))
}

pub fn load(app: &AppHandle) -> Result<Vec<OperationTransaction>, AppError> {
    let path = history_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let bytes = fs::read(&path).map_err(|error| AppError::PersistenceFailed {
        message: format!("Unable to read transaction history: {error}"),
    })?;
    serde_json::from_slice(&bytes).map_err(|error| AppError::PersistenceFailed {
        message: format!("Transaction history is invalid: {error}"),
    })
}

pub fn record(
    app: &AppHandle,
    transaction: OperationTransaction,
) -> Result<Vec<OperationTransaction>, AppError> {
    let path = history_path(app)?;
    let mut transactions = if path.exists() {
        load(app)?
    } else {
        Vec::new()
    };
    if let Some(existing) = transactions
        .iter_mut()
        .find(|item| item.id == transaction.id)
    {
        *existing = transaction;
    } else {
        transactions.insert(0, transaction);
    }
    transactions.truncate(MAX_TRANSACTIONS);
    let encoded =
        serde_json::to_vec_pretty(&transactions).map_err(|error| AppError::PersistenceFailed {
            message: format!("Unable to encode transaction history: {error}"),
        })?;
    let temporary_path = path.with_extension("json.tmp");
    fs::write(&temporary_path, encoded).map_err(|error| AppError::PersistenceFailed {
        message: format!("Unable to write transaction history: {error}"),
    })?;
    if path.exists() {
        fs::remove_file(&path).map_err(|error| AppError::PersistenceFailed {
            message: format!("Unable to replace transaction history: {error}"),
        })?;
    }
    fs::rename(&temporary_path, &path).map_err(|error| AppError::PersistenceFailed {
        message: format!("Unable to finalize transaction history: {error}"),
    })?;
    Ok(transactions)
}
