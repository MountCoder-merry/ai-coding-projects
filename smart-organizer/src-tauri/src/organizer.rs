use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::errors::AppError;
use crate::scanner::{FileCategory, ScanResult};

/// A single filesystem change in an organization plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileOperation {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationType {
    Move,
    Rename,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationStatus {
    Pending,
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationPlanSummary {
    pub operation_count: usize,
    pub total_bytes: u64,
    pub new_folder_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationPlan {
    pub id: String,
    pub root_path: String,
    pub created_at: String,
    pub operations: Vec<FileOperation>,
    pub summary: OrganizationPlanSummary,
}

/// Build a preview-only plan. No filesystem contents are changed here.
pub fn generate_plan(root: &str, scan: &ScanResult) -> Result<OrganizationPlan, AppError> {
    let trimmed = root.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidPath {
            message: "A folder path is required".to_string(),
        });
    }

    let root_path = PathBuf::from(trimmed);
    let root_metadata =
        fs::metadata(&root_path).map_err(|error| map_root_error(&root_path, error))?;
    if !root_metadata.is_dir() {
        return Err(AppError::NotDirectory {
            path: trimmed.to_string(),
        });
    }

    let created_at = now_timestamp();
    let plan_id = format!("plan-{}", created_at.replace([':', '-'], ""));
    let mut operations = Vec::new();
    let mut target_folders = HashSet::new();
    let mut planned_destinations = HashSet::new();

    for item in &scan.items {
        if item.is_directory || item.category == FileCategory::Other {
            continue;
        }

        let source = PathBuf::from(&item.path);
        let Some(source_name) = source.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        // A scan is expected to contain direct children. Rejecting a path that
        // does not belong to the selected root prevents a forged scan from
        // causing a plan to move an unrelated file.
        if !same_direct_child(&root_path, &source, source_name) {
            continue;
        }

        let folder_name = category_folder(item.category);
        let target_folder = root_path.join(folder_name);
        if same_path(source.parent(), Some(&target_folder)) {
            continue;
        }

        let preferred_destination = target_folder.join(source_name);
        let destination = unique_destination(&preferred_destination, &mut planned_destinations);
        if source == destination {
            continue;
        }

        let source_string = source.to_string_lossy().into_owned();
        let destination_string = destination.to_string_lossy().into_owned();
        let id = format!("{}-{}", plan_id, operations.len() + 1);
        operations.push(FileOperation {
            id,
            operation_type: OperationType::Move,
            source: source_string,
            destination: destination_string,
            reason: format!("Move to {folder_name}"),
            status: OperationStatus::Pending,
            error: None,
            size_bytes: item.size_bytes,
        });
        target_folders.insert(target_folder);
    }

    let total_bytes = operations
        .iter()
        .map(|operation| operation.size_bytes)
        .sum();
    let summary = OrganizationPlanSummary {
        operation_count: operations.len(),
        total_bytes,
        new_folder_count: target_folders.len(),
    };

    Ok(OrganizationPlan {
        id: plan_id,
        root_path: trimmed.to_string(),
        created_at,
        operations,
        summary,
    })
}

fn category_folder(category: FileCategory) -> &'static str {
    match category {
        FileCategory::Images => "Images",
        FileCategory::Videos => "Videos",
        FileCategory::Documents => "Documents",
        FileCategory::Archives => "Archives",
        FileCategory::Audio => "Audio",
        FileCategory::Code => "Code",
        FileCategory::Applications => "Applications",
        FileCategory::Other => "Other",
    }
}

fn unique_destination(preferred: &Path, planned: &mut HashSet<PathBuf>) -> PathBuf {
    if !preferred.exists() && !planned.contains(preferred) {
        let result = preferred.to_path_buf();
        planned.insert(result.clone());
        return result;
    }

    let parent = match preferred.parent() {
        Some(parent) => parent,
        None => Path::new(""),
    };
    let stem = match preferred.file_stem().and_then(|value| value.to_str()) {
        Some(stem) => stem,
        None => "file",
    };
    let extension = match preferred.extension().and_then(|value| value.to_str()) {
        Some(value) => format!(".{value}"),
        None => String::new(),
    };

    let mut suffix = 1usize;
    loop {
        let candidate = parent.join(format!("{stem} ({suffix}){extension}"));
        if !candidate.exists() && !planned.contains(&candidate) {
            planned.insert(candidate.clone());
            return candidate;
        }
        suffix += 1;
    }
}

fn same_direct_child(root: &Path, source: &Path, source_name: &str) -> bool {
    let Some(parent) = source.parent() else {
        return false;
    };
    same_path(Some(parent), Some(root))
        && source.file_name().and_then(|name| name.to_str()) == Some(source_name)
}

fn same_path(left: Option<&Path>, right: Option<&Path>) -> bool {
    let (Some(left), Some(right)) = (left, right) else {
        return false;
    };
    if left == right {
        return true;
    }
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

fn map_root_error(path: &Path, error: std::io::Error) -> AppError {
    match error.kind() {
        std::io::ErrorKind::NotFound => AppError::PathNotFound {
            path: path.to_string_lossy().into_owned(),
        },
        std::io::ErrorKind::PermissionDenied => AppError::PathNotReadable {
            path: path.to_string_lossy().into_owned(),
        },
        _ => AppError::ScanFailed {
            path: path.to_string_lossy().into_owned(),
            message: error.to_string(),
        },
    }
}

pub(crate) fn now_timestamp() -> String {
    let seconds = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() as i64,
        Err(_) => 0,
    };
    let days = seconds.div_euclid(86_400);
    let day_seconds = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = day_seconds / 3_600;
    let minute = day_seconds % 3_600 / 60;
    let second = day_seconds % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let shifted = days_since_epoch + 719_468;
    let era = (if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    }) / 146_097;
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_part = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_part + 2) / 5 + 1;
    let month = month_part + if month_part < 10 { 3 } else { -9 };
    (year + if month <= 2 { 1 } else { 0 }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{ScanSummary, ScannedFile};
    use std::fs::{self, File};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("smart-organizer-plan-{}-{id}", std::process::id()));
            fs::create_dir_all(&path).expect("tempdir");
            Self(path)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn scan(root: &Path, items: Vec<ScannedFile>) -> ScanResult {
        ScanResult {
            root: root.to_string_lossy().into_owned(),
            scanned_at: now_timestamp(),
            items,
            summary: ScanSummary {
                total_files: 0,
                total_bytes: 0,
                skipped_entries: 0,
                by_category: FileCategory::ALL
                    .into_iter()
                    .map(|category| (category, 0))
                    .collect(),
            },
        }
    }

    fn item(root: &Path, name: &str, category: FileCategory, size_bytes: u64) -> ScannedFile {
        ScannedFile {
            id: root.join(name).to_string_lossy().into_owned(),
            name: name.to_string(),
            path: root.join(name).to_string_lossy().into_owned(),
            extension: Path::new(name)
                .extension()
                .and_then(|value| value.to_str())
                .map(str::to_string),
            category,
            size_bytes,
            created_at: None,
            modified_at: None,
            is_directory: false,
        }
    }

    #[test]
    fn creates_category_plan_and_skips_other_and_directories() {
        let temp = TestDir::new();
        File::create(temp.0.join("photo.jpg")).expect("photo");
        let plan = generate_plan(
            temp.0.to_str().expect("path"),
            &scan(
                &temp.0,
                vec![
                    item(&temp.0, "photo.jpg", FileCategory::Images, 12),
                    item(&temp.0, "unknown.bin", FileCategory::Other, 2),
                    ScannedFile {
                        is_directory: true,
                        ..item(&temp.0, "folder", FileCategory::Documents, 0)
                    },
                ],
            ),
        )
        .expect("plan");
        assert_eq!(plan.operations.len(), 1);
        assert_eq!(
            plan.operations[0].destination,
            temp.0.join("Images").join("photo.jpg").to_string_lossy()
        );
        assert_eq!(plan.summary.total_bytes, 12);
        assert_eq!(plan.summary.new_folder_count, 1);
    }

    #[test]
    fn picks_suffix_for_existing_and_planned_conflicts() {
        let temp = TestDir::new();
        fs::create_dir(temp.0.join("Images")).expect("images");
        File::create(temp.0.join("Images/photo.jpg")).expect("existing");
        File::create(temp.0.join("photo.jpg")).expect("photo");
        File::create(temp.0.join("photo-copy.jpg")).expect("copy");
        let mut destinations = HashSet::new();
        let first = unique_destination(&temp.0.join("Images/photo.jpg"), &mut destinations);
        let second = unique_destination(&temp.0.join("Images/photo.jpg"), &mut destinations);
        assert_eq!(first, temp.0.join("Images/photo (1).jpg"));
        assert_eq!(second, temp.0.join("Images/photo (2).jpg"));
    }
}
