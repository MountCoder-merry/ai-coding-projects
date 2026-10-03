use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::scanner::FileCategory;

const CATEGORY_CATALOG: &str = include_str!("../resources/category_catalog.json");

static CATALOG: OnceLock<BTreeMap<String, Vec<String>>> = OnceLock::new();

fn catalog() -> &'static BTreeMap<String, Vec<String>> {
    CATALOG.get_or_init(|| serde_json::from_str(CATEGORY_CATALOG).unwrap_or_default())
}

fn category_name(category: FileCategory) -> &'static str {
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

/// Classify an extension using the bundled catalog, keeping rule data outside
/// the scanner and UI code. Unknown or absent extensions become `Other`.
pub fn classify_extension(extension: Option<&str>) -> FileCategory {
    let Some(extension) = extension.map(str::to_ascii_lowercase) else {
        return FileCategory::Other;
    };
    FileCategory::ALL
        .into_iter()
        .find(|category| {
            catalog()
                .get(category_name(*category))
                .is_some_and(|extensions| extensions.iter().any(|item| item == &extension))
        })
        .unwrap_or(FileCategory::Other)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_from_catalog_case_insensitively() {
        assert_eq!(classify_extension(Some("JpG")), FileCategory::Images);
        assert_eq!(classify_extension(Some("PDF")), FileCategory::Documents);
        assert_eq!(classify_extension(Some("unknown")), FileCategory::Other);
        assert_eq!(classify_extension(None), FileCategory::Other);
    }
}
