# Holocron

A CLI learning assistant powered by Claude Code. Learn topics, analyze articles, and generate TIL entries or knowledge notes.

## Installation

```bash
cargo install --git https://github.com/jspauley/holocron
```

Or from source:

```bash
git clone https://github.com/jspauley/holocron
cd holocron
cargo install --path .
```

## Quick Start

```bash
# First run prompts for configuration
holocron

# Or initialize a TIL repo and configure manually
holocron init ~/my-til
holocron config --til-path ~/my-til --notes-path ~/notes
```

### Example Session

```
holocron> /learn rust ownership

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
```

## Commands

| Command | Description |
|---------|-------------|
| `holocron` | Start interactive mode |
| `holocron learn <topic>` | Deep dive on a topic |
| `holocron link <url>` | Analyze an article |
| `holocron init <path>` | Initialize new TIL repo |
| `holocron config` | View/update configuration |
| `/learn <topic>` | Interactive: start deep dive |
| `/link <url>` | Interactive: analyze URL |
| `/til` | Interactive: generate TIL entry |
| `/note` | Interactive: generate knowledge note |
| `/exit` | Interactive: exit |

## Configuration

Config stored at `~/.config/holocron/config.toml`:

```bash
holocron config --til-path ~/til
holocron config --notes-path ~/obsidian/notes
holocron config --notes-format obsidian  # or: logseq, plain
holocron config --archive-dir archive    # TIL subdirectory name
```

## Requirements

- Rust 1.70+
- [Claude Code](https://claude.ai/code) CLI installed and authenticated

## License

MIT
