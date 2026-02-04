use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "holocron")]
#[command(author, version, about = "A learning assistant CLI backed by Claude Code")]
#[command(long_about = "Holocron is your personal learning companion. Start an interactive \
    session to deep dive into topics, analyze articles, and generate TIL entries or detailed notes.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start a deep dive learning session on a topic
    Learn {
        /// The topic to learn about
        topic: String,

        /// Category for TIL generation (e.g., git, rust, sql)
        #[arg(short, long)]
        category: Option<String>,
    },

    /// Analyze and summarize an article from a URL
    Link {
        /// The URL to analyze
        url: String,

        /// Category for TIL generation (e.g., git, rust, sql)
        #[arg(short, long)]
        category: Option<String>,
    },

    /// Initialize a new TIL repository
    Init {
        /// Path where the TIL repository should be created
        path: PathBuf,
    },

    /// View or update holocron configuration
    Config {
        /// Set the TIL repository path
        #[arg(long)]
        til_path: Option<PathBuf>,

        /// Set the notes repository path
        #[arg(long)]
        notes_path: Option<PathBuf>,

        /// Set the notes format (obsidian, logseq, plain)
        #[arg(long)]
        notes_format: Option<String>,

        /// Set the archive directory name
        #[arg(long)]
        archive_dir: Option<String>,
    },
}
