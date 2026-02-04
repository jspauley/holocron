use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

const TIL_SKILL: &str = r#"# /til - Generate TIL Entry

Generate a short "Today I Learned" markdown entry based on the current conversation.

## Format
- H1 title (action-oriented: "Update A Forked Repo", "Create Table In Postgres")
- 1-2 sentence intro explaining WHEN/WHY you'd do this
- Code example(s) with brief prose transitions
- **Target: 15-30 lines total. Shorter is better.**

## Style
- Conversational: "If you want to...", "you can...", "let's..."
- NO H2/H3 sections - just flowing prose and code
- NO lists of options, categories, or variations
- NO comprehensive coverage - just the one thing you learned
- End casually ("That's it!") or with one practical tip

## Output
Return ONLY the markdown. No preamble.

## Example (25 lines)
```markdown
# Update A Forked Repo

If you have forked a repo, and the original now includes changes you'd like to have in your copy, you can pull in the latest version using `git rebase`.

First, add the original repo as an upstream source:

\`\`\`bash
git remote add upstream https://github.com/original-source/repo-name.git
\`\`\`

Next, fetch all of the branches and rebase:

\`\`\`bash
git fetch upstream
git rebase upstream/main
\`\`\`

That's it! You may need to force push if this branch was already on GitHub.
```
"#;

const NOTE_SKILL: &str = r#"# /note - Generate Knowledge Base Note

Generate a comprehensive knowledge base note based on the current conversation. This is for personal knowledge management systems like Obsidian or Logseq.

## Format Requirements

### Frontmatter (YAML)
```yaml
---
title: [Descriptive title]
date: [YYYY-MM-DD]
tags: [relevant, tags, as, list]
aliases: [alternative, names]
---
```

### Content Structure
1. **Title** (H1) - Clear, descriptive title
2. **Overview** - 2-3 paragraph introduction explaining the concept
3. **Key Concepts** - Detailed breakdown of important ideas
4. **Examples** - Comprehensive code examples with explanations
5. **Common Patterns** - Typical use cases and patterns
6. **Gotchas & Tips** - Things to watch out for, best practices
7. **Session Q&A** (optional) - Key questions and answers from our conversation
8. **Related Topics** - Links to related concepts using `[[wiki-link]]` format
9. **Sources** (if from URL analysis) - Original sources consulted

## Style Guidelines
- Write in a teaching tone, as if explaining to a colleague
- Include plenty of code examples with annotations
- Be thorough - this is for deep understanding, not quick reference
- Use bullet points and headers for scannability
- Include practical, real-world applications
- Target length: 100-300 lines

## Output Format
Return ONLY the markdown content for the note file, starting with the YAML frontmatter. Do not include any preamble or explanation.
"#;

const CLAUDE_SETTINGS: &str = r#"{
  "permissions": {
    "allowedTools": ["WebFetch", "WebSearch"]
  }
}
"#;

const README_TEMPLATE: &str = r#"# Today I Learned

A collection of concise write-ups on things I learn day to day.

0 TILs & Counting

---

### Categories

---
"#;

/// Initialize a new TIL repository at the given path
pub fn init_til_repo(path: &Path, archive_dir: &str) -> Result<()> {
    // Create main directory
    fs::create_dir_all(path)
        .with_context(|| format!("Failed to create TIL directory: {:?}", path))?;

    // Create archive directory
    let archive_path = path.join(archive_dir);
    fs::create_dir_all(&archive_path)
        .with_context(|| format!("Failed to create archive directory: {:?}", archive_path))?;

    // Create .claude/commands directory
    let commands_path = path.join(".claude").join("commands");
    fs::create_dir_all(&commands_path)
        .with_context(|| format!("Failed to create commands directory: {:?}", commands_path))?;

    // Write README.md
    let readme_path = path.join("README.md");
    if !readme_path.exists() {
        fs::write(&readme_path, README_TEMPLATE)
            .with_context(|| "Failed to write README.md")?;
    }

    // Write /til skill
    let til_skill_path = commands_path.join("til.md");
    fs::write(&til_skill_path, TIL_SKILL)
        .with_context(|| "Failed to write til.md skill")?;

    // Write /note skill
    let note_skill_path = commands_path.join("note.md");
    fs::write(&note_skill_path, NOTE_SKILL)
        .with_context(|| "Failed to write note.md skill")?;

    // Write settings.json with allowed tools
    let settings_path = path.join(".claude").join("settings.json");
    fs::write(&settings_path, CLAUDE_SETTINGS)
        .with_context(|| "Failed to write settings.json")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_til_repo() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let til_path = temp_dir.path().join("my-til");

        init_til_repo(&til_path, "archive")?;

        assert!(til_path.join("README.md").exists());
        assert!(til_path.join("archive").exists());
        assert!(til_path.join(".claude/commands/til.md").exists());
        assert!(til_path.join(".claude/commands/note.md").exists());
        assert!(til_path.join(".claude/settings.json").exists());

        Ok(())
    }
}
