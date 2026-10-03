use std::collections::BTreeMap;
use std::fs::{self, DirEntry, Metadata};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::errors::AppError;

/// Categories used by the scanner and exposed to the TypeScript client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FileCategory {
    Images,
    Videos,
    Documents,
    Archives,
    Audio,
    Code,
    Applications,
    Other,
}

impl FileCategory {
    pub const ALL: [Self; 8] = [
        Self::Images,
        Self::Videos,
        Self::Documents,
        Self::Archives,
        Self::Audio,
        Self::Code,
        Self::Applications,
        Self::Other,
    ];
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedFile {
    pub id: String,
    pub name: String,
    pub path: String,
    pub extension: Option<String>,
    pub category: FileCategory,
    pub size_bytes: u64,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
    pub is_directory: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    pub total_files: usize,
    pub total_bytes: u64,
    pub skipped_entries: usize,
    pub by_category: BTreeMap<FileCategory, usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub root: String,
    pub scanned_at: String,
    pub items: Vec<ScannedFile>,
    pub summary: ScanSummary,
}

/// Scan only the immediate children of `path`. No file is opened or changed.
pub fn scan_folder(path: &str) -> Result<ScanResult, AppError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidPath {
            message: "A folder path is required".to_string(),
        });
    }

    let root = PathBuf::from(trimmed);
    let root_metadata = fs::metadata(&root).map_err(|error| map_path_error(&root, error))?;
    if !root_metadata.is_dir() {
        return Err(AppError::NotDirectory {
            path: trimmed.to_string(),
        });
    }

    // Opening the directory up front makes unreadable roots fail as an error instead
    // of returning an apparently successful empty scan.
    let entries = fs::read_dir(&root).map_err(|error| map_path_error(&root, error))?;
    let mut entries = entries
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| map_path_error(&root, error))?;
    entries.sort_by(|left, right| left.file_name().cmp(&right.file_name()));

    let mut items = Vec::new();
    let mut skipped_entries = 0;
    let mut by_category = empty_category_counts();

    for entry in entries {
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => {
                skipped_entries += 1;
                continue;
            }
        };

        // Never follow links. This also prevents a directory symlink from making
        // the direct-child MVP recurse into another tree by accident.
        if file_type.is_symlink() || file_type.is_dir() {
            skipped_entries += 1;
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => {
                skipped_entries += 1;
                continue;
            }
        };
        if is_hidden(&entry, &metadata) || !file_type.is_file() {
            skipped_entries += 1;
            continue;
        }

        let item = scanned_file(&entry, &metadata);
        *by_category.entry(item.category).or_default() += 1;
        items.push(item);
    }

    let total_bytes = items.iter().map(|item| item.size_bytes).sum();
    let total_files = items.len();
    Ok(ScanResult {
        root: trimmed.to_string(),
        scanned_at: format_timestamp(SystemTime::now()),
        items,
        summary: ScanSummary {
            total_files,
            total_bytes,
            skipped_entries,
            by_category,
        },
    })
}

fn empty_category_counts() -> BTreeMap<FileCategory, usize> {
    FileCategory::ALL
        .into_iter()
        .map(|category| (category, 0))
        .collect()
}

fn scanned_file(entry: &DirEntry, metadata: &Metadata) -> ScannedFile {
    let path = entry.path();
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_lowercase());
    let category = crate::rules::classify_extension(extension.as_deref());
    let path_string = path.to_string_lossy().into_owned();

    ScannedFile {
        id: path_string.clone(),
        name: entry.file_name().to_string_lossy().into_owned(),
        path: path_string,
        extension,
        category,
        size_bytes: metadata.len(),
        created_at: metadata.created().ok().map(format_timestamp),
        modified_at: metadata.modified().ok().map(format_timestamp),
        is_directory: false,
    }
}

fn is_hidden(entry: &DirEntry, metadata: &Metadata) -> bool {
    let name_is_hidden = entry.file_name().to_string_lossy().starts_with('.');

    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
        const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;
        return name_is_hidden
            || metadata.file_attributes() & (FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM) != 0;
    }

    #[cfg(not(windows))]
    {
        let _ = metadata;
        name_is_hidden
    }
}

fn map_path_error(path: &Path, error: std::io::Error) -> AppError {
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

fn format_timestamp(time: SystemTime) -> String {
    let seconds = time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let days = seconds.div_euclid(86_400);
    let day_seconds = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = day_seconds / 3_600;
    let minute = day_seconds % 3_600 / 60;
    let second = day_seconds % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

// Howard Hinnant's civil-date conversion, kept local to avoid another runtime
// dependency just for formatting filesystem timestamps.
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
    use std::fs::{self, File};
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("smart-organizer-test-{}-{id}", std::process::id()));
            fs::create_dir_all(&path).expect("tempdir");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn scans_sorted_direct_files_and_populates_summary() {
        let temp = TestDir::new();
        let root = temp.path();
        let mut image = File::create(root.join("z-photo.JPG")).expect("image");
        image.write_all(b"123").expect("write image");
        drop(image);
        File::create(root.join("a-notes.md")).expect("notes");
        File::create(root.join("unknown.bin")).expect("unknown");
        fs::create_dir(root.join("nested")).expect("nested");
        File::create(root.join(".hidden.txt")).expect("hidden");

        let result = scan_folder(root.to_str().expect("utf8 path")).expect("scan");
        let names = result
            .items
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["a-notes.md", "unknown.bin", "z-photo.JPG"]);
        assert_eq!(result.summary.total_files, 3);
        assert_eq!(result.summary.total_bytes, 3);
        assert_eq!(result.summary.skipped_entries, 2);
        assert_eq!(result.summary.by_category[&FileCategory::Documents], 1);
        assert_eq!(result.summary.by_category[&FileCategory::Images], 1);
        assert_eq!(result.summary.by_category[&FileCategory::Other], 1);
        assert!(result.items.iter().all(|item| !item.is_directory));
    }

    #[test]
    fn rejects_empty_missing_and_file_paths() {
        assert!(matches!(
            scan_folder("  "),
            Err(AppError::InvalidPath { .. })
        ));
        assert!(matches!(
            scan_folder("this-path-does-not-exist"),
            Err(AppError::PathNotFound { .. })
        ));

        let temp = TestDir::new();
        let file = temp.path().join("file.txt");
        File::create(&file).expect("file");
        assert!(matches!(
            scan_folder(file.to_str().expect("utf8 path")),
            Err(AppError::NotDirectory { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn skips_symlinks() {
        use std::os::unix::fs::symlink;

        let temp = TestDir::new();
        File::create(temp.path().join("real.txt")).expect("real");
        symlink(temp.path().join("real.txt"), temp.path().join("link.txt")).expect("symlink");

        let result = scan_folder(temp.path().to_str().expect("utf8 path")).expect("scan");
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.summary.skipped_entries, 1);
    }
}
