# Holocron

A learning assistant CLI backed by Claude Code. Like a Jedi Holocron, it stores and helps you access knowledge through interactive learning sessions.

## Features

- **Deep Dive Mode**: Explain any topic in technical detail with follow-up Q&A
- **Link Mode**: Fetch and analyze articles, blog posts, or documentation
- **TIL Generation**: Create concise "Today I Learned" entries (`/til`)
- **Knowledge Notes**: Generate comprehensive notes for Obsidian/Logseq (`/note`)
- **Configurable**: Set up TIL repo, notes repo, and customize paths

## Installation

```bash
cargo build --release

# Add to your PATH
cp target/release/holocron ~/.local/bin/
```

## Quick Start

```bash
# First run will prompt for configuration
holocron

# Or initialize a new TIL repo
holocron init ~/my-til

# Configure paths
holocron config --til-path ~/my-til --notes-path ~/notes
```

## Usage

### Interactive Mode

Just run `holocron`:

```
════════════════════════════════════════════════════════════
  HOLOCRON - Your Learning Assistant
════════════════════════════════════════════════════════════

Commands:
  /deep <topic>  - Start a deep dive on a topic
  /link <url>    - Analyze an article from URL
  /til           - Generate TIL from session
  /note          - Generate detailed note
  /exit          - Exit holocron
```

### Example Session

```
holocron> /deep rust ownership

[Claude explains Rust ownership in detail...]

holocron> What about lifetimes?

[Claude explains lifetimes...]

holocron> /til

Generated TIL:
────────────────────────────────────
# Understanding Rust Ownership
...
────────────────────────────────────

Save as rust/understanding_rust_ownership.md? Yes, save it
✓ TIL saved to: ~/til/archive/rust/understanding_rust_ownership.md

holocron> /note

Generated Note:
────────────────────────────────────
---
title: Rust Ownership and Lifetimes
date: 2024-02-03
tags: [rust, memory, ownership]
---
...
────────────────────────────────────

Save as rust_ownership_and_lifetimes.md? Yes, save it
✓ Note saved to: ~/notes/rust_ownership_and_lifetimes.md
```

### Direct Commands

```bash
# Start a deep dive session
holocron learn "kubernetes networking" --category devops

# Analyze an article
holocron link "https://example.com/article" --category web
```

## Configuration

Config is stored at `~/.config/holocron/config.toml`:

```toml
til_path = "/path/to/til"
archive_dir = "archive"
notes_path = "/path/to/notes"  # optional
notes_format = "obsidian"       # obsidian, logseq, or plain
```

### Commands

```bash
# View current config
holocron config

# Update settings
holocron config --til-path ~/new-til
holocron config --notes-path ~/obsidian/knowledge
holocron config --notes-format logseq
holocron config --archive-dir entries
```

## Output Formats

### TIL (`/til`)
Concise, 10-30 line entries for quick reference:
- H1 title
- Brief introduction
- Code examples
- Key takeaway

### Notes (`/note`)
Comprehensive knowledge base entries:
- YAML frontmatter (title, date, tags, aliases)
- Detailed overview
- Multiple code examples with annotations
- Session Q&A highlights
- Related topics as wiki-links
- Sources (for link mode)

## Commands Reference

| Command | Description |
|---------|-------------|
| `holocron` | Start interactive mode |
| `holocron learn <topic>` | Deep dive on a topic |
| `holocron link <url>` | Analyze an article |
| `holocron init <path>` | Initialize new TIL repo |
| `holocron config` | View/update configuration |

### Interactive Commands

| Command | Description |
|---------|-------------|
| `/deep <topic>` | Start deep dive session |
| `/link <url>` | Analyze URL |
| `/til` | Generate TIL entry |
| `/note` | Generate knowledge note |
| `/exit` | Exit holocron |

## Requirements

- Rust 1.70+
- Claude Code CLI installed and authenticated

## Development

```bash
cargo test
cargo clippy -- -D warnings
cargo build --release
```

## License

MIT
