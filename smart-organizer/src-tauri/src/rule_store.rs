use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::errors::AppError;
use crate::rule_engine::{validate_rule, StructuredRule};

const RULES_FILE: &str = "rules.json";
const MAX_RULES: usize = 100;

fn rules_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| AppError::PersistenceFailed {
            message: format!("Unable to resolve the app data directory: {error}"),
        })?;
    fs::create_dir_all(&directory).map_err(|error| AppError::PersistenceFailed {
        message: format!("Unable to create the app data directory: {error}"),
    })?;
    Ok(directory.join(RULES_FILE))
}

pub fn load(app: &AppHandle) -> Result<Vec<StructuredRule>, AppError> {
    let path = rules_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let bytes = fs::read(path).map_err(|error| AppError::PersistenceFailed {
        message: format!("Unable to read saved rules: {error}"),
    })?;
    serde_json::from_slice(&bytes).map_err(|error| AppError::PersistenceFailed {
        message: format!("Saved rules are invalid: {error}"),
    })
}

pub fn save(app: &AppHandle, rule: StructuredRule) -> Result<Vec<StructuredRule>, AppError> {
    validate_rule(&rule)?;
    let path = rules_path(app)?;
    let mut rules = if path.exists() {
        load(app)?
    } else {
        Vec::new()
    };
    if let Some(existing) = rules.iter_mut().find(|item| item.id == rule.id) {
        *existing = rule;
    } else {
        rules.insert(0, rule);
    }
    rules.truncate(MAX_RULES);
    let encoded =
        serde_json::to_vec_pretty(&rules).map_err(|error| AppError::PersistenceFailed {
            message: format!("Unable to encode saved rules: {error}"),
        })?;
    let temporary_path = path.with_extension("json.tmp");
    fs::write(&temporary_path, encoded).map_err(|error| AppError::PersistenceFailed {
        message: format!("Unable to write saved rules: {error}"),
    })?;
    if path.exists() {
        fs::remove_file(&path).map_err(|error| AppError::PersistenceFailed {
            message: format!("Unable to replace saved rules: {error}"),
        })?;
    }
    fs::rename(&temporary_path, &path).map_err(|error| AppError::PersistenceFailed {
        message: format!("Unable to finalize saved rules: {error}"),
    })?;
    Ok(rules)
}
