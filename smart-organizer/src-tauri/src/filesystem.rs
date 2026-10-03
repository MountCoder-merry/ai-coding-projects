use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::errors::AppError;
use crate::organizer::{
    now_timestamp, FileOperation, OperationStatus, OperationType, OrganizationPlan,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutedOperation {
    pub id: String,
    #[serde(rename = "type")]
    pub operation_type: OperationType,
    pub source: String,
    pub destination: String,
    pub reason: String,
    pub status: OperationStatus,
    pub error: Option<String>,
    pub size_bytes: u64,
}

impl From<FileOperation> for ExecutedOperation {
    fn from(operation: FileOperation) -> Self {
        Self {
            id: operation.id,
            operation_type: operation.operation_type,
            source: operation.source,
            destination: operation.destination,
            reason: operation.reason,
            status: operation.status,
            error: operation.error,
            size_bytes: operation.size_bytes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationTransaction {
    pub id: String,
    pub created_at: String,
    pub root_path: String,
    pub operations: Vec<ExecutedOperation>,
    pub completed_count: usize,
    pub failed_count: usize,
    pub undone_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResult {
    pub transaction: OperationTransaction,
    pub completed_count: usize,
    pub failed_count: usize,
}

/// The filesystem boundary for plan application and undo.
#[derive(Debug, Default, Clone, Copy)]
pub struct FileOperationEngine;

impl FileOperationEngine {
    pub fn new() -> Self {
        Self
    }

    /// Apply every operation independently. A failed operation never prevents
    /// later operations from being attempted, and is retained in the result.
    pub fn apply(&self, plan: &OrganizationPlan) -> Result<ApplyResult, AppError> {
        let root = validate_root(&plan.root_path)?;
        let mut operations = Vec::with_capacity(plan.operations.len());
        let mut completed_count = 0usize;
        let mut failed_count = 0usize;

        for operation in &plan.operations {
            let mut executed = ExecutedOperation::from(operation.clone());
            executed.status = OperationStatus::Pending;
            executed.error = None;

            let result = apply_operation(&root, operation);
            match result {
                Ok(()) => {
                    executed.status = OperationStatus::Success;
                    completed_count += 1;
                }
                Err(error) => {
                    executed.status = OperationStatus::Failed;
                    executed.error = Some(error.to_string());
                    failed_count += 1;
                }
            }
            operations.push(executed);
        }

        Ok(ApplyResult {
            transaction: OperationTransaction {
                id: format!("transaction-{}", plan.id),
                created_at: now_timestamp(),
                root_path: plan.root_path.clone(),
                operations,
                completed_count,
                failed_count,
                undone_at: None,
            },
            completed_count,
            failed_count,
        })
    }

    /// Restore only operations that completed successfully. Existing source
    /// paths are treated as conflicts and are never overwritten.
    pub fn undo(&self, transaction: &OperationTransaction) -> Result<ApplyResult, AppError> {
        let root = validate_root(&transaction.root_path)?;
        let mut operations = transaction.operations.clone();
        let mut completed_count = 0usize;
        let mut failed_count = 0usize;

        for operation in &mut operations {
            if operation.status != OperationStatus::Success {
                continue;
            }

            operation.status = OperationStatus::Pending;
            operation.error = None;
            let source = PathBuf::from(&operation.destination);
            let destination = PathBuf::from(&operation.source);
            let result = validate_operation_paths(&root, &source, &destination)
                .and_then(|_| move_without_overwrite(&source, &destination));

            match result {
                Ok(()) => {
                    operation.status = OperationStatus::Success;
                    completed_count += 1;
                }
                Err(error) => {
                    operation.status = OperationStatus::Failed;
                    operation.error = Some(error.to_string());
                    failed_count += 1;
                }
            }
        }

        Ok(ApplyResult {
            transaction: OperationTransaction {
                id: transaction.id.clone(),
                created_at: transaction.created_at.clone(),
                root_path: transaction.root_path.clone(),
                operations,
                completed_count,
                failed_count,
                undone_at: Some(now_timestamp()),
            },
            completed_count,
            failed_count,
        })
    }
}

pub fn apply_plan(plan: &OrganizationPlan) -> Result<ApplyResult, AppError> {
    FileOperationEngine::new().apply(plan)
}

pub fn undo_transaction(transaction: &OperationTransaction) -> Result<ApplyResult, AppError> {
    FileOperationEngine::new().undo(transaction)
}

fn validate_root(root: &str) -> Result<PathBuf, AppError> {
    let trimmed = root.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidPath {
            message: "A folder path is required".to_string(),
        });
    }
    let path = PathBuf::from(trimmed);
    let metadata = fs::metadata(&path).map_err(|error| map_operation_error(&path, error))?;
    if !metadata.is_dir() {
        return Err(AppError::NotDirectory {
            path: trimmed.to_string(),
        });
    }
    fs::canonicalize(&path).map_err(|error| map_operation_error(&path, error))
}

fn apply_operation(root: &Path, operation: &FileOperation) -> Result<(), AppError> {
    let source = PathBuf::from(&operation.source);
    let destination = PathBuf::from(&operation.destination);
    validate_operation_paths(root, &source, &destination)?;
    if !source.exists() {
        return Err(AppError::PathNotFound {
            path: source.to_string_lossy().into_owned(),
        });
    }
    if !source.is_file() {
        return Err(AppError::InvalidPath {
            message: format!(
                "The source is not a regular file: {}",
                source.to_string_lossy()
            ),
        });
    }

    let parent = destination.parent().ok_or_else(|| AppError::InvalidPath {
        message: format!(
            "The destination has no parent: {}",
            destination.to_string_lossy()
        ),
    })?;
    fs::create_dir_all(parent).map_err(|error| map_operation_error(parent, error))?;
    let canonical_parent =
        fs::canonicalize(parent).map_err(|error| map_operation_error(parent, error))?;
    if !canonical_parent.starts_with(root) {
        return Err(AppError::InvalidPath {
            message: format!(
                "The destination is outside the selected root: {}",
                destination.to_string_lossy()
            ),
        });
    }
    move_without_overwrite(&source, &destination)
}

fn validate_operation_paths(
    root: &Path,
    source: &Path,
    destination: &Path,
) -> Result<(), AppError> {
    let source_canonical = match fs::canonicalize(source) {
        Ok(path) => path,
        Err(error) => return Err(map_operation_error(source, error)),
    };
    if !source_canonical.starts_with(root) {
        return Err(AppError::InvalidPath {
            message: format!(
                "The source is outside the selected root: {}",
                source.to_string_lossy()
            ),
        });
    }

    let parent = destination.parent().ok_or_else(|| AppError::InvalidPath {
        message: format!(
            "The destination has no parent: {}",
            destination.to_string_lossy()
        ),
    })?;
    if parent.exists() {
        let canonical_parent =
            fs::canonicalize(parent).map_err(|error| map_operation_error(parent, error))?;
        if !canonical_parent.starts_with(root) {
            return Err(AppError::InvalidPath {
                message: format!(
                    "The destination is outside the selected root: {}",
                    destination.to_string_lossy()
                ),
            });
        }
    } else {
        let nearest = nearest_existing_ancestor(parent).ok_or_else(|| AppError::InvalidPath {
            message: format!(
                "The destination has no valid parent: {}",
                destination.to_string_lossy()
            ),
        })?;
        let canonical_nearest =
            fs::canonicalize(&nearest).map_err(|error| map_operation_error(&nearest, error))?;
        if !canonical_nearest.starts_with(root) {
            return Err(AppError::InvalidPath {
                message: format!(
                    "The destination is outside the selected root: {}",
                    destination.to_string_lossy()
                ),
            });
        }
    }
    Ok(())
}

fn nearest_existing_ancestor(path: &Path) -> Option<PathBuf> {
    let mut current = path;
    loop {
        if current.exists() {
            return Some(current.to_path_buf());
        }
        current = current.parent()?;
    }
}

/// Move a regular file without ever opening an existing destination for write.
/// A hard link is preferred (same-volume and metadata-preserving); the fallback
/// uses `create_new` and copies bytes before removing the source.
fn move_without_overwrite(source: &Path, destination: &Path) -> Result<(), AppError> {
    if destination.exists() {
        return Err(AppError::InvalidPath {
            message: format!(
                "The destination already exists: {}",
                destination.to_string_lossy()
            ),
        });
    }

    match fs::hard_link(source, destination) {
        Ok(()) => {
            if let Err(error) = fs::remove_file(source) {
                let _ = fs::remove_file(destination);
                return Err(map_operation_error(source, error));
            }
            Ok(())
        }
        Err(link_error) => {
            copy_then_remove(source, destination).map_err(|copy_error| AppError::InvalidPath {
                message: format!(
                    "Unable to move {} to {} (link: {}; copy: {})",
                    source.to_string_lossy(),
                    destination.to_string_lossy(),
                    link_error,
                    copy_error
                ),
            })
        }
    }
}

fn copy_then_remove(source: &Path, destination: &Path) -> io::Result<()> {
    let mut input = File::open(source)?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    let copy_result = io::copy(&mut input, &mut output);
    if let Err(error) = copy_result {
        let _ = fs::remove_file(destination);
        return Err(error);
    }
    output.flush()?;
    drop(output);
    if let Err(error) = fs::remove_file(source) {
        let _ = fs::remove_file(destination);
        return Err(error);
    }
    Ok(())
}

fn map_operation_error(path: &Path, error: io::Error) -> AppError {
    match error.kind() {
        io::ErrorKind::NotFound => AppError::PathNotFound {
            path: path.to_string_lossy().into_owned(),
        },
        io::ErrorKind::PermissionDenied => AppError::PathNotReadable {
            path: path.to_string_lossy().into_owned(),
        },
        _ => AppError::ScanFailed {
            path: path.to_string_lossy().into_owned(),
            message: error.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::organizer::{OperationStatus, OperationType, OrganizationPlanSummary};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("smart-organizer-fs-{}-{id}", std::process::id()));
            fs::create_dir_all(&path).expect("tempdir");
            Self(path)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn plan(root: &Path, operations: Vec<FileOperation>) -> OrganizationPlan {
        OrganizationPlan {
            id: "test-plan".to_string(),
            root_path: root.to_string_lossy().into_owned(),
            created_at: now_timestamp(),
            summary: OrganizationPlanSummary {
                operation_count: operations.len(),
                total_bytes: operations
                    .iter()
                    .map(|operation| operation.size_bytes)
                    .sum(),
                new_folder_count: 1,
            },
            operations,
        }
    }

    fn operation(root: &Path, name: &str, destination: &str) -> FileOperation {
        FileOperation {
            id: name.to_string(),
            operation_type: OperationType::Move,
            source: root.join(name).to_string_lossy().into_owned(),
            destination: root.join(destination).to_string_lossy().into_owned(),
            reason: "test".to_string(),
            status: OperationStatus::Pending,
            error: None,
            size_bytes: 3,
        }
    }

    #[test]
    fn applies_and_undoes_successfully() {
        let temp = TestDir::new();
        fs::write(temp.0.join("note.txt"), b"abc").expect("source");
        let result = FileOperationEngine::new()
            .apply(&plan(
                &temp.0,
                vec![operation(&temp.0, "note.txt", "Documents/note.txt")],
            ))
            .expect("apply");
        assert_eq!(result.completed_count, 1);
        assert!(!temp.0.join("note.txt").exists());
        assert_eq!(
            fs::read(temp.0.join("Documents/note.txt")).expect("destination"),
            b"abc"
        );

        let undone = FileOperationEngine::new()
            .undo(&result.transaction)
            .expect("undo");
        assert_eq!(undone.completed_count, 1);
        assert!(temp.0.join("note.txt").exists());
        assert!(!temp.0.join("Documents/note.txt").exists());
    }

    #[test]
    fn conflict_is_failed_without_overwrite() {
        let temp = TestDir::new();
        fs::create_dir_all(temp.0.join("Documents")).expect("folder");
        fs::write(temp.0.join("note.txt"), b"new").expect("source");
        fs::write(temp.0.join("Documents/note.txt"), b"old").expect("destination");
        let result = FileOperationEngine::new()
            .apply(&plan(
                &temp.0,
                vec![operation(&temp.0, "note.txt", "Documents/note.txt")],
            ))
            .expect("apply result");
        assert_eq!(result.completed_count, 0);
        assert_eq!(result.failed_count, 1);
        assert_eq!(fs::read(temp.0.join("note.txt")).expect("source"), b"new");
        assert_eq!(
            fs::read(temp.0.join("Documents/note.txt")).expect("destination"),
            b"old"
        );
        assert_eq!(
            result.transaction.operations[0].status,
            OperationStatus::Failed
        );
    }

    #[test]
    fn partial_failure_keeps_success_and_failure_records() {
        let temp = TestDir::new();
        fs::write(temp.0.join("ok.txt"), b"ok").expect("source");
        fs::write(temp.0.join("bad.txt"), b"bad").expect("source");
        fs::create_dir_all(temp.0.join("Documents")).expect("folder");
        fs::write(temp.0.join("Documents/bad.txt"), b"existing").expect("destination");
        let result = FileOperationEngine::new()
            .apply(&plan(
                &temp.0,
                vec![
                    operation(&temp.0, "ok.txt", "Documents/ok.txt"),
                    operation(&temp.0, "bad.txt", "Documents/bad.txt"),
                ],
            ))
            .expect("apply result");
        assert_eq!(result.completed_count, 1);
        assert_eq!(result.failed_count, 1);
        assert_eq!(
            result.transaction.operations[0].status,
            OperationStatus::Success
        );
        assert_eq!(
            result.transaction.operations[1].status,
            OperationStatus::Failed
        );
    }

    #[test]
    fn undo_conflict_does_not_overwrite_existing_source() {
        let temp = TestDir::new();
        fs::write(temp.0.join("note.txt"), b"source").expect("source");
        let result = FileOperationEngine::new()
            .apply(&plan(
                &temp.0,
                vec![operation(&temp.0, "note.txt", "Documents/note.txt")],
            ))
            .expect("apply");
        fs::write(temp.0.join("note.txt"), b"newer").expect("conflict");
        let undone = FileOperationEngine::new()
            .undo(&result.transaction)
            .expect("undo result");
        assert_eq!(undone.completed_count, 0);
        assert_eq!(undone.failed_count, 1);
        assert_eq!(fs::read(temp.0.join("note.txt")).expect("source"), b"newer");
        assert!(temp.0.join("Documents/note.txt").exists());
    }
}
