use crate::errors::AppError;
use crate::filesystem::{ApplyResult, OperationTransaction};
use crate::history;
use crate::organizer::OrganizationPlan;
use crate::rule_engine::StructuredRule;
use crate::rule_store;
use crate::scanner::ScanResult;
use serde::Serialize;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub engine_status: &'static str,
}

#[tauri::command]
pub fn health_check() -> Result<String, AppError> {
    Ok("ready".to_string())
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        name: "Smart Organizer",
        version: "0.1.0",
        engine_status: "ready",
    }
}

#[tauri::command]
pub fn scan_folder(root: String) -> Result<ScanResult, AppError> {
    crate::scanner::scan_folder(&root)
}

#[tauri::command]
pub fn get_desktop_folder(app: AppHandle) -> Result<String, AppError> {
    app.path()
        .desktop_dir()
        .map(|path| path.to_string_lossy().into_owned())
        .map_err(|error| AppError::PathNotReadable {
            path: format!("Desktop ({error})"),
        })
}

#[tauri::command]
pub fn generate_plan(root: String, scan: ScanResult) -> Result<OrganizationPlan, AppError> {
    crate::organizer::generate_plan(&root, &scan)
}

#[tauri::command]
pub fn apply_plan(app: AppHandle, plan: OrganizationPlan) -> Result<ApplyResult, AppError> {
    let result = crate::filesystem::apply_plan(&plan)?;
    history::record(&app, result.transaction.clone())?;
    Ok(result)
}

#[tauri::command]
pub fn undo_transaction(
    app: AppHandle,
    transaction: OperationTransaction,
) -> Result<ApplyResult, AppError> {
    let result = crate::filesystem::undo_transaction(&transaction)?;
    history::record(&app, result.transaction.clone())?;
    Ok(result)
}

#[tauri::command]
pub fn load_history(app: AppHandle) -> Result<Vec<OperationTransaction>, AppError> {
    history::load(&app)
}

#[tauri::command]
pub fn validate_rule(rule: StructuredRule) -> Result<(), AppError> {
    crate::rule_engine::validate_rule(&rule)
}

#[tauri::command]
pub fn parse_rule_mock(input: String) -> Result<StructuredRule, AppError> {
    crate::rule_engine::parse_natural_language_mock(&input)
}

#[tauri::command]
pub fn load_rules(app: AppHandle) -> Result<Vec<StructuredRule>, AppError> {
    rule_store::load(&app)
}

#[tauri::command]
pub fn save_rule(app: AppHandle, rule: StructuredRule) -> Result<Vec<StructuredRule>, AppError> {
    rule_store::save(&app, rule)
}
