use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR: &str = "holocron";
const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path to the TIL repository
    pub til_path: PathBuf,

    /// Directory within til_path for TIL entries (default: "archive")
    #[serde(default = "default_archive_dir")]
    pub archive_dir: String,

    /// Optional path to notes/knowledge base repository
    pub notes_path: Option<PathBuf>,

    /// Notes format: obsidian, logseq, or plain
    #[serde(default = "default_notes_format")]
    pub notes_format: NotesFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NotesFormat {
    #[default]
    Obsidian,
    Logseq,
    Plain,
}

impl std::fmt::Display for NotesFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotesFormat::Obsidian => write!(f, "obsidian"),
            NotesFormat::Logseq => write!(f, "logseq"),
            NotesFormat::Plain => write!(f, "plain"),
        }
    }
}

fn default_archive_dir() -> String {
    "archive".to_string()
}

fn default_notes_format() -> NotesFormat {
    NotesFormat::Obsidian
}

#[allow(dead_code)]
impl Config {
    /// Load config from the default location
    pub fn load() -> Result<Option<Self>> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config from {:?}", config_path))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| "Failed to parse config file")?;

        Ok(Some(config))
    }

    /// Save config to the default location
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory {:?}", parent))?;
        }

        let content = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;

        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config to {:?}", config_path))?;

        Ok(())
    }

    /// Get the config file path
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?;

        Ok(config_dir.join(CONFIG_DIR).join(CONFIG_FILE))
    }

    /// Check if config exists
    pub fn exists() -> Result<bool> {
        Ok(Self::config_path()?.exists())
    }

    /// Create a new config with the given TIL path
    pub fn new(til_path: PathBuf) -> Self {
        Self {
            til_path,
            archive_dir: default_archive_dir(),
            notes_path: None,
            notes_format: default_notes_format(),
        }
    }

    /// Get the full path to the archive directory
    pub fn archive_path(&self) -> PathBuf {
        self.til_path.join(&self.archive_dir)
    }

    /// Get the TIL skill path (within the TIL repo)
    pub fn til_skill_path(&self) -> PathBuf {
        self.til_path.join(".claude").join("commands").join("til.md")
    }

    /// Get the note skill path (within the TIL repo)
    pub fn note_skill_path(&self) -> PathBuf {
        self.til_path.join(".claude").join("commands").join("note.md")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = Config {
            til_path: PathBuf::from("/path/to/til"),
            archive_dir: "archive".to_string(),
            notes_path: Some(PathBuf::from("/path/to/notes")),
            notes_format: NotesFormat::Obsidian,
        };

        let toml_str = toml::to_string_pretty(&config).expect("serialize");
        let parsed: Config = toml::from_str(&toml_str).expect("deserialize");

        assert_eq!(parsed.til_path, config.til_path);
        assert_eq!(parsed.archive_dir, config.archive_dir);
    }

    #[test]
    fn test_default_values() {
        let toml_str = r#"til_path = "/path/to/til""#;
        let config: Config = toml::from_str(toml_str).expect("deserialize");

        assert_eq!(config.archive_dir, "archive");
    }

    #[test]
    fn test_notes_format_display() {
        assert_eq!(format!("{}", NotesFormat::Obsidian), "obsidian");
        assert_eq!(format!("{}", NotesFormat::Logseq), "logseq");
        assert_eq!(format!("{}", NotesFormat::Plain), "plain");
    }

    #[test]
    fn test_notes_format_default() {
        let format = NotesFormat::default();
        assert!(matches!(format, NotesFormat::Obsidian));
    }

    #[test]
    fn test_config_new() {
        let config = Config::new(PathBuf::from("/test/path"));

        assert_eq!(config.til_path, PathBuf::from("/test/path"));
        assert_eq!(config.archive_dir, "archive");
        assert!(config.notes_path.is_none());
        assert!(matches!(config.notes_format, NotesFormat::Obsidian));
    }

    #[test]
    fn test_config_archive_path() {
        let config = Config {
            til_path: PathBuf::from("/test/til"),
            archive_dir: "entries".to_string(),
            notes_path: None,
            notes_format: NotesFormat::Plain,
        };

        assert_eq!(config.archive_path(), PathBuf::from("/test/til/entries"));
    }

    #[test]
    fn test_config_skill_paths() {
        let config = Config::new(PathBuf::from("/test/til"));

        assert_eq!(
            config.til_skill_path(),
            PathBuf::from("/test/til/.claude/commands/til.md")
        );
        assert_eq!(
            config.note_skill_path(),
            PathBuf::from("/test/til/.claude/commands/note.md")
        );
    }

    #[test]
    fn test_config_with_all_notes_formats() {
        for (format_str, expected) in [
            ("obsidian", NotesFormat::Obsidian),
            ("logseq", NotesFormat::Logseq),
            ("plain", NotesFormat::Plain),
        ] {
            let toml_str = format!(
                r#"til_path = "/path"
notes_format = "{}""#,
                format_str
            );
            let config: Config = toml::from_str(&toml_str).expect("deserialize");
            assert_eq!(format!("{}", config.notes_format), format!("{}", expected));
        }
    }

    #[test]
    fn test_config_path_exists() {
        // This test just ensures config_path() doesn't panic
        let result = Config::config_path();
        assert!(result.is_ok());
    }
}
