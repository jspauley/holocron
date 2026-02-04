use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Write a note to the notes repository
pub fn write_note(notes_path: &Path, filename: &str, content: &str) -> Result<PathBuf> {
    // Create notes directory if it doesn't exist
    if !notes_path.exists() {
        fs::create_dir_all(notes_path)
            .with_context(|| format!("Failed to create notes directory: {:?}", notes_path))?;
    }

    let filename = sanitize_filename(filename);
    let file_path = notes_path.join(&filename);

    // Ensure trailing newline
    let content = ensure_trailing_newline(content);
    fs::write(&file_path, &content)
        .with_context(|| format!("Failed to write note file: {:?}", file_path))?;

    Ok(file_path)
}

/// Extract title from note content (from frontmatter or first H1)
pub fn extract_title(content: &str) -> Option<String> {
    // First try to get from frontmatter
    if content.starts_with("---") {
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() >= 2 {
            for line in parts[1].lines() {
                let line = line.trim();
                if line.starts_with("title:") {
                    let title = line.strip_prefix("title:").map(|s| s.trim());
                    if let Some(t) = title {
                        // Remove quotes if present
                        let t = t.trim_matches('"').trim_matches('\'');
                        if !t.is_empty() {
                            return Some(t.to_string());
                        }
                    }
                }
            }
        }
    }

    // Fall back to first H1
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix("# ") {
            return Some(title.trim().to_string());
        }
    }

    None
}

/// Generate a filename from a title
pub fn title_to_filename(title: &str) -> String {
    let filename: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();

    let result = collapse_underscores(&filename);
    format!("{}.md", result)
}

fn collapse_underscores(s: &str) -> String {
    let mut result = String::new();
    let mut last_was_underscore = false;

    for c in s.chars() {
        if c == '_' {
            if !last_was_underscore {
                result.push(c);
            }
            last_was_underscore = true;
        } else {
            result.push(c);
            last_was_underscore = false;
        }
    }

    result.trim_matches('_').to_string()
}

fn ensure_trailing_newline(s: &str) -> String {
    if s.ends_with('\n') {
        s.to_string()
    } else {
        format!("{}\n", s)
    }
}

fn sanitize_filename(filename: &str) -> String {
    let name = if filename.ends_with(".md") {
        filename.to_string()
    } else {
        format!("{}.md", filename)
    };

    name.replace(' ', "_").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_extract_title_from_frontmatter() {
        let content = r#"---
title: My Note Title
date: 2024-01-01
tags: [test]
---

# Content Here
"#;
        assert_eq!(extract_title(content), Some("My Note Title".to_string()));
    }

    #[test]
    fn test_extract_title_from_frontmatter_quoted() {
        let content = r#"---
title: "Quoted Title"
---
"#;
        assert_eq!(extract_title(content), Some("Quoted Title".to_string()));
    }

    #[test]
    fn test_extract_title_from_frontmatter_single_quoted() {
        let content = r#"---
title: 'Single Quoted'
---
"#;
        assert_eq!(extract_title(content), Some("Single Quoted".to_string()));
    }

    #[test]
    fn test_extract_title_from_h1() {
        let content = "# My H1 Title\n\nSome content.";
        assert_eq!(extract_title(content), Some("My H1 Title".to_string()));
    }

    #[test]
    fn test_extract_title_no_title() {
        let content = "Some content without a title.";
        assert_eq!(extract_title(content), None);
    }

    #[test]
    fn test_extract_title_empty_frontmatter_title() {
        let content = r#"---
title:
---
# Fallback Title
"#;
        assert_eq!(extract_title(content), Some("Fallback Title".to_string()));
    }

    #[test]
    fn test_title_to_filename() {
        assert_eq!(title_to_filename("Rust Clippy"), "rust_clippy.md");
        assert_eq!(title_to_filename("Git: The Basics"), "git_the_basics.md");
    }

    #[test]
    fn test_title_to_filename_special_chars() {
        assert_eq!(title_to_filename("What's New?"), "what_s_new.md");
        assert_eq!(title_to_filename("Test & More"), "test_more.md");
    }

    #[test]
    fn test_collapse_underscores() {
        assert_eq!(collapse_underscores("a__b"), "a_b");
        assert_eq!(collapse_underscores("___test___"), "test");
        assert_eq!(collapse_underscores("no_change"), "no_change");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test"), "test.md");
        assert_eq!(sanitize_filename("test.md"), "test.md");
        assert_eq!(sanitize_filename("Test File"), "test_file.md");
        assert_eq!(sanitize_filename("UPPER.md"), "upper.md");
    }

    #[test]
    fn test_write_note() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let content = "# Test Note\n\nContent here.";

        let path = write_note(temp_dir.path(), "test_note.md", content)?;

        assert!(path.exists());
        // Should have trailing newline added
        assert_eq!(fs::read_to_string(&path)?, format!("{}\n", content));

        Ok(())
    }

    #[test]
    fn test_write_note_creates_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let notes_path = temp_dir.path().join("new_notes_dir");

        let path = write_note(&notes_path, "note.md", "# Note")?;

        assert!(path.exists());
        assert!(notes_path.exists());

        Ok(())
    }

    #[test]
    fn test_write_note_without_extension() -> Result<()> {
        let temp_dir = TempDir::new()?;

        let path = write_note(temp_dir.path(), "my_note", "# Note")?;

        assert!(path.to_string_lossy().ends_with(".md"));

        Ok(())
    }
}
