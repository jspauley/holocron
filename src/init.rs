use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

const TIL_SKILL: &str = r#"# /til - Generate TIL Entry

Generate a "Today I Learned" markdown entry based on the current conversation.

## Format Requirements
- H1 title describing what was learned (use sentence case, be specific)
- 1-2 sentence introduction providing context for why this is useful
- Practical code examples in fenced blocks with language specified
- Concise: 10-30 lines typically
- Match existing TIL style in this repository

## Style Guidelines
- Active voice, present tense
- Technically accurate but accessible
- Include specific commands/code that solve the problem
- No fluff - get straight to the useful information
- End with a brief note about when to use this or what to watch out for

## Output Format
Return ONLY the markdown content for the TIL file. Do not include any preamble or explanation - just the raw markdown that should be written to the file.

## Example Structure
```markdown
# How to Do the Thing

Brief context about when and why you'd want to do this thing.

\`\`\`bash
the-actual-command --with flags
\`\`\`

Optional additional explanation or follow-up commands if needed.
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

        Ok(())
    }
}
