use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Write a TIL to the appropriate category folder and update the README
pub fn write_til(
    repo_root: &Path,
    archive_dir: &str,
    category: &str,
    filename: &str,
    content: &str,
    title: &str,
) -> Result<PathBuf> {
    let category_lower = category.to_lowercase();
    let category_dir = repo_root.join(archive_dir).join(&category_lower);

    // Create category directory if it doesn't exist
    if !category_dir.exists() {
        fs::create_dir_all(&category_dir)
            .with_context(|| format!("Failed to create category directory: {:?}", category_dir))?;
    }

    // Write the TIL file (ensure trailing newline)
    let filename = sanitize_filename(filename);
    let file_path = category_dir.join(&filename);
    let content = ensure_trailing_newline(content);
    fs::write(&file_path, &content)
        .with_context(|| format!("Failed to write TIL file: {:?}", file_path))?;

    // Update README.md
    update_readme(repo_root, archive_dir, &category_lower, &filename, title)?;

    Ok(file_path)
}

/// Extract title from TIL markdown content (first H1 heading)
pub fn extract_title(content: &str) -> Option<String> {
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

    // Remove consecutive underscores and trim
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

fn sanitize_filename(filename: &str) -> String {
    let name = if filename.ends_with(".md") {
        filename.to_string()
    } else {
        format!("{}.md", filename)
    };

    name.replace(' ', "_").to_lowercase()
}

fn update_readme(
    repo_root: &Path,
    archive_dir: &str,
    category: &str,
    filename: &str,
    title: &str,
) -> Result<()> {
    let readme_path = repo_root.join("README.md");
    let content = fs::read_to_string(&readme_path).context("Failed to read README.md")?;

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // Update TIL count
    update_til_count(&mut lines);

    // Find or create category section and add entry
    add_entry_to_category(&mut lines, archive_dir, category, filename, title)?;

    // Write back (ensure trailing newline)
    let new_content = format!("{}\n", lines.join("\n"));
    fs::write(&readme_path, new_content).context("Failed to write README.md")?;

    Ok(())
}

fn update_til_count(lines: &mut [String]) {
    for line in lines.iter_mut() {
        // Match lines like "25 TILs & Counting"
        if line.contains("TILs & Counting") {
            if let Some(count_str) = line.split_whitespace().next() {
                if let Ok(count) = count_str.parse::<u32>() {
                    *line = format!("{} TILs & Counting", count + 1);
                    return;
                }
            }
        }
    }
}

fn add_entry_to_category(
    lines: &mut Vec<String>,
    archive_dir: &str,
    category: &str,
    filename: &str,
    title: &str,
) -> Result<()> {
    let category_header = format!("### {}", capitalize_first(category));
    let entry = format!("- [{}]({}/{}/{})", title, archive_dir, category, filename);

    // Find the category section
    let category_idx = find_category_index(lines, &category_header, category);

    if let Some(idx) = category_idx {
        let insert_idx = find_insertion_point(lines, idx);
        lines.insert(insert_idx, entry);
    } else {
        add_new_category(lines, archive_dir, category, filename, title)?;
    }

    Ok(())
}

fn find_category_index(lines: &[String], category_header: &str, category: &str) -> Option<usize> {
    for (i, line) in lines.iter().enumerate() {
        if line.trim().eq_ignore_ascii_case(category_header)
            || line.trim().to_lowercase() == format!("### {}", category.to_lowercase())
        {
            return Some(i);
        }
    }
    None
}

fn find_insertion_point(lines: &[String], category_idx: usize) -> usize {
    let mut insert_idx = category_idx + 1;

    while insert_idx < lines.len() {
        let line = &lines[insert_idx];
        if line.starts_with("###") || line.starts_with("---") {
            break;
        }
        if line.starts_with("- [") || line.trim().is_empty() {
            insert_idx += 1;
        } else {
            break;
        }
    }

    // Insert before empty line or next section
    if insert_idx > 0 && lines[insert_idx - 1].trim().is_empty() {
        insert_idx - 1
    } else {
        insert_idx
    }
}

fn add_new_category(
    lines: &mut Vec<String>,
    archive_dir: &str,
    category: &str,
    filename: &str,
    title: &str,
) -> Result<()> {
    let category_display = capitalize_first(category);

    // Add to Categories list (find ### Categories section)
    if let Some(end_idx) = find_categories_end(lines) {
        let cat_link = format!("* [{}](#{})", category_display, category.to_lowercase());
        lines.insert(end_idx, cat_link);
    }

    // Add the category section at the end
    let insert_pos = find_end_position(lines);

    lines.insert(insert_pos, String::new());
    lines.insert(insert_pos + 1, format!("### {}", category_display));
    lines.insert(insert_pos + 2, String::new());
    lines.insert(
        insert_pos + 3,
        format!("- [{}]({}/{}/{})", title, archive_dir, category, filename),
    );
    lines.insert(insert_pos + 4, String::new());

    Ok(())
}

fn find_categories_end(lines: &[String]) -> Option<usize> {
    let mut in_categories = false;

    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "### Categories" {
            in_categories = true;
        } else if in_categories
            && (line.starts_with("---")
                || (line.starts_with("###") && line.trim() != "### Categories"))
        {
            return Some(i);
        }
    }
    None
}

fn find_end_position(lines: &[String]) -> usize {
    let mut insert_pos = lines.len();
    while insert_pos > 0 && lines[insert_pos - 1].trim().is_empty() {
        insert_pos -= 1;
    }
    insert_pos
}

fn ensure_trailing_newline(s: &str) -> String {
    if s.ends_with('\n') {
        s.to_string()
    } else {
        format!("{}\n", s)
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_title_to_filename() {
        assert_eq!(title_to_filename("Git Rebasing"), "git_rebasing.md");
        assert_eq!(
            title_to_filename("How to Use --onto"),
            "how_to_use_onto.md"
        );
        assert_eq!(
            title_to_filename("Command Line Reset"),
            "command_line_reset.md"
        );
    }

    #[test]
    fn test_extract_title() {
        let content = "# My Title\n\nSome content here.";
        assert_eq!(extract_title(content), Some("My Title".to_string()));

        let content = "  # Trimmed Title  \n\nMore content.";
        assert_eq!(extract_title(content), Some("Trimmed Title".to_string()));

        let content = "No heading here";
        assert_eq!(extract_title(content), None);
    }

    #[test]
    fn test_collapse_underscores() {
        assert_eq!(collapse_underscores("a__b___c"), "a_b_c");
        assert_eq!(collapse_underscores("___test___"), "test");
        assert_eq!(collapse_underscores("no_change"), "no_change");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test"), "test.md");
        assert_eq!(sanitize_filename("test.md"), "test.md");
        assert_eq!(sanitize_filename("Test File"), "test_file.md");
    }

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("git"), "Git");
        assert_eq!(capitalize_first("rust"), "Rust");
        assert_eq!(capitalize_first(""), "");
    }

    #[test]
    fn test_update_til_count() {
        let mut lines = vec![
            "# TIL".to_string(),
            "25 TILs & Counting".to_string(),
            "other".to_string(),
        ];
        update_til_count(&mut lines);
        assert_eq!(lines[1], "26 TILs & Counting");
    }

    #[test]
    fn test_find_categories_end() {
        let lines = vec![
            "### Categories".to_string(),
            "* [Git](#git)".to_string(),
            "---".to_string(),
        ];
        assert_eq!(find_categories_end(&lines), Some(2));
    }

    #[test]
    fn test_write_til() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let readme_content = r#"# TIL
5 TILs & Counting
### Categories
* [Git](#git)
---
### Git
- [Existing Entry](archive/git/existing.md)
"#;
        fs::write(temp_dir.path().join("README.md"), readme_content)?;

        let result = write_til(
            temp_dir.path(),
            "archive",
            "git",
            "new_entry.md",
            "# New Entry\n\nContent here.",
            "New Entry",
        )?;

        assert!(result.exists());
        assert!(result.to_string_lossy().contains("archive/git"));
        let readme = fs::read_to_string(temp_dir.path().join("README.md"))?;
        assert!(readme.contains("6 TILs & Counting"));
        assert!(readme.contains("- [New Entry](archive/git/new_entry.md)"));

        Ok(())
    }

    #[test]
    fn test_write_til_new_category() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let readme_content = r#"# TIL
5 TILs & Counting
### Categories
* [Git](#git)
---
### Git
- [Existing Entry](archive/git/existing.md)
"#;
        fs::write(temp_dir.path().join("README.md"), readme_content)?;

        let result = write_til(
            temp_dir.path(),
            "archive",
            "rust",
            "ownership.md",
            "# Ownership\n\nRust ownership.",
            "Ownership",
        )?;

        assert!(result.exists());
        assert!(result.to_string_lossy().contains("archive/rust"));
        let readme = fs::read_to_string(temp_dir.path().join("README.md"))?;
        assert!(readme.contains("### Rust"));
        assert!(readme.contains("- [Ownership](archive/rust/ownership.md)"));

        Ok(())
    }
}
