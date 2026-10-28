//! File › Add to Oracle Library: the active document goes into the library ("Mes créations") as an
//! Oracle Layout document, as it is now (saved or not); the library draws its thumbnail with
//! Oracle Layout.

use crate::DesignApp;

/// Characters a file name can't hold on Windows (and `/` anywhere).
fn file_name_safe(name: &str) -> String {
    let cleaned: String = name.chars().map(|c| if matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || c.is_control() { '-' } else { c }).collect();
    let trimmed = cleaned.trim().trim_end_matches('.').to_string();
    if trimmed.is_empty() { "Document".into() } else { trimmed }
}

/// Adds the active document to the library.
pub fn add_active(app: &mut DesignApp) -> Result<(), String> {
    to_library(app, true).map(|_| ())
}

/// Stores the active document in the library without showing it (Send to goes through the library):
/// where it is now.
pub fn store_active(app: &mut DesignApp) -> Result<std::path::PathBuf, String> {
    to_library(app, false)?.into_iter().next().ok_or_else(|| "the document couldn't be stored in the Oracle Library".to_string())
}

fn to_library(app: &mut DesignApp, show: bool) -> Result<Vec<std::path::PathBuf>, String> {
    let st = app.session.active().ok_or("no document is open")?;
    let stem = st.path.as_deref().and_then(|p| std::path::Path::new(p).file_stem()).map_or_else(|| "Document".to_string(), |s| s.to_string_lossy().into_owned());
    let bytes = oracle_layout_format::save(&st.doc).map_err(|e| e.to_string())?;
    let dir = std::env::temp_dir().join(format!("oracle-layout-to-library-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let file = dir.join(format!("{}.oracle-layout", file_name_safe(&stem)));
    std::fs::write(&file, bytes).map_err(|e| format!("{}: {e}", file.display()))?;
    let added = if show { app.oracle_library.add_files(std::slice::from_ref(&file)) } else { app.oracle_library.store_files(std::slice::from_ref(&file)) };
    let _ = std::fs::remove_file(&file); // Copied into the library; a leftover is only a temp file.
    Ok(added)
}

#[cfg(test)]
mod tests {
    use super::file_name_safe;

    #[test]
    fn names_become_file_names() {
        assert_eq!(file_name_safe("Charte: v1?"), "Charte- v1-");
        assert_eq!(file_name_safe(" "), "Document");
    }
}
