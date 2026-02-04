mod claude;
mod cli;
mod config;
mod init;
mod modes;
mod notes;
mod session;
mod til;

use anyhow::{anyhow, Result};
use clap::Parser;
use cli::{Cli, Commands};
use colored::*;
use config::{Config, NotesFormat};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use modes::{build_deep_dive_prompt, build_link_prompt};
use session::{LearningMode, Session};
use std::io::{self, Write};
use std::path::PathBuf;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init { path }) => {
            run_init(path)?;
        }
        Some(Commands::Config {
            til_path,
            notes_path,
            notes_format,
            archive_dir,
        }) => {
            run_config(til_path, notes_path, notes_format, archive_dir)?;
        }
        Some(Commands::Learn { topic, category }) => {
            let config = ensure_config()?;
            let mode = LearningMode::DeepDive {
                topic: topic.clone(),
            };
            let session = Session::new(mode, category);
            run_learning_session(session, build_deep_dive_prompt(&topic), &config)?;
        }
        Some(Commands::Link { url, category }) => {
            let config = ensure_config()?;
            let mode = LearningMode::Link { url: url.clone() };
            let session = Session::new(mode, category);
            run_learning_session(session, build_link_prompt(&url), &config)?;
        }
        None => {
            let config = ensure_config()?;
            run_interactive_mode(&config)?;
        }
    }

    Ok(())
}

/// Ensure config exists, running first-time setup if needed
fn ensure_config() -> Result<Config> {
    if let Some(config) = Config::load()? {
        return Ok(config);
    }

    // First-time setup
    println!("{}", "═".repeat(60).bright_cyan());
    println!(
        "{}",
        "  Welcome to Holocron!  ".bold().bright_cyan()
    );
    println!("{}", "═".repeat(60).bright_cyan());
    println!();
    println!("Let's set up your configuration.");
    println!();

    let til_path: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Path to your TIL repository")
        .interact_text()?;

    let til_path = PathBuf::from(shellexpand::tilde(&til_path).to_string());

    // Check if it exists, offer to create or install skills
    if !til_path.exists() {
        let create = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("TIL repository doesn't exist. Create it?")
            .items(&["Yes, initialize a new TIL repo", "No, I'll create it manually"])
            .default(0)
            .interact()?;

        if create == 0 {
            init::init_til_repo(&til_path, "archive")?;
            println!("{} Created TIL repository at {:?}", "✓".green(), til_path);
        }
    } else if !til_path.join(".claude").join("commands").exists() {
        // Existing repo without skills - offer to install them
        let install = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Install Claude Code skills (/til, /note) in this repo?")
            .items(&["Yes, install skills", "No, skip"])
            .default(0)
            .interact()?;

        if install == 0 {
            init::init_til_repo(&til_path, "archive")?;
            println!("{} Installed skills at {:?}", "✓".green(), til_path.join(".claude/commands"));
        }
    }

    // Ask about notes path
    println!();
    let setup_notes = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Set up a notes/knowledge base path? (for Obsidian, Logseq, etc.)")
        .items(&["Yes", "No, skip for now"])
        .default(1)
        .interact()?;

    let mut config = Config::new(til_path);

    if setup_notes == 0 {
        let notes_path: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Path to your notes repository")
            .interact_text()?;

        config.notes_path = Some(PathBuf::from(shellexpand::tilde(&notes_path).to_string()));

        let format = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Notes format")
            .items(&["Obsidian", "Logseq", "Plain markdown"])
            .default(0)
            .interact()?;

        config.notes_format = match format {
            0 => NotesFormat::Obsidian,
            1 => NotesFormat::Logseq,
            _ => NotesFormat::Plain,
        };
    }

    config.save()?;

    let config_path = Config::config_path()?;
    println!();
    println!("{} Config saved to {:?}", "✓".green(), config_path);
    println!();

    Ok(config)
}

/// Run the init command
fn run_init(path: PathBuf) -> Result<()> {
    let path = PathBuf::from(shellexpand::tilde(path.to_string_lossy().as_ref()).to_string());

    let readme_existed = path.join("README.md").exists();
    let archive_existed = path.join("archive").exists();

    init::init_til_repo(&path, "archive")?;

    println!("{} Initialized TIL repository at {:?}", "✓".green(), path);
    println!();
    println!("Created:");
    if readme_existed {
        println!("  - README.md {}", "(already existed, skipped)".dimmed());
    } else {
        println!("  - README.md");
    }
    if archive_existed {
        println!("  - archive/ {}", "(already existed)".dimmed());
    } else {
        println!("  - archive/");
    }
    println!("  - .claude/commands/til.md");
    println!("  - .claude/commands/note.md");
    println!("  - .claude/settings.json");
    println!();
    println!("Run {} to set this as your TIL path.", "holocron config --til-path <path>".cyan());

    Ok(())
}

/// Run the config command
fn run_config(
    til_path: Option<PathBuf>,
    notes_path: Option<PathBuf>,
    notes_format: Option<String>,
    archive_dir: Option<String>,
) -> Result<()> {
    let mut config = Config::load()?.unwrap_or_else(|| Config::new(PathBuf::new()));
    let mut changed = false;

    if let Some(path) = til_path {
        config.til_path = PathBuf::from(shellexpand::tilde(path.to_string_lossy().as_ref()).to_string());
        changed = true;
    }

    if let Some(path) = notes_path {
        config.notes_path = Some(PathBuf::from(shellexpand::tilde(path.to_string_lossy().as_ref()).to_string()));
        changed = true;
    }

    if let Some(format) = notes_format {
        config.notes_format = match format.to_lowercase().as_str() {
            "obsidian" => NotesFormat::Obsidian,
            "logseq" => NotesFormat::Logseq,
            "plain" => NotesFormat::Plain,
            _ => return Err(anyhow!("Invalid notes format. Use: obsidian, logseq, or plain")),
        };
        changed = true;
    }

    if let Some(dir) = archive_dir {
        config.archive_dir = dir;
        changed = true;
    }

    if changed {
        config.save()?;
        println!("{} Configuration updated.", "✓".green());
    }

    // Display current config
    println!();
    println!("{}", "Current Configuration:".bold());
    println!("  TIL path:     {:?}", config.til_path);
    println!("  Archive dir:  {}", config.archive_dir);
    if let Some(ref notes) = config.notes_path {
        println!("  Notes path:   {:?}", notes);
        println!("  Notes format: {}", config.notes_format);
    } else {
        println!("  Notes path:   (not configured)");
    }
    println!();
    println!("Config file: {:?}", Config::config_path()?);

    Ok(())
}

fn print_welcome_banner() {
    println!("{}", "═".repeat(60).bright_cyan());
    println!(
        "{}",
        "  HOLOCRON - Your Learning Assistant  "
            .bold()
            .bright_cyan()
    );
    println!("{}", "═".repeat(60).bright_cyan());
    println!();
    println!("Commands:");
    println!(
        "  {} - Start a deep dive on a topic",
        "/learn <topic>".green()
    );
    println!("  {}    - Analyze an article from URL", "/link <url>".green());
    println!("  {}          - Generate TIL from session", "/til".green());
    println!("  {}         - Generate detailed note", "/note".green());
    println!("  {}         - Exit holocron", "/exit".green());
    println!();
    println!("Or just type to continue the conversation.");
    println!();
}

fn run_interactive_mode(config: &Config) -> Result<()> {
    print_welcome_banner();

    let mut session: Option<Session> = None;

    loop {
        let input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("holocron")
            .allow_empty(false)
            .interact_text()?;

        let input = input.trim();

        if input.eq_ignore_ascii_case("/quit") || input.eq_ignore_ascii_case("/exit") {
            println!("{}", "May the Force be with you.".bright_cyan());
            break;
        }

        if let Some(handled) = handle_command(input, &mut session, config)? {
            if handled {
                continue;
            }
        }

        // Regular conversation continuation
        if let Some(ref mut sess) = session {
            send_and_display(input, sess)?;
        } else {
            println!(
                "{}",
                "Start a session with /learn <topic> or /link <url>".yellow()
            );
        }
    }

    Ok(())
}

fn handle_command(input: &str, session: &mut Option<Session>, config: &Config) -> Result<Option<bool>> {
    if let Some(topic) = input.strip_prefix("/learn ") {
        let topic = topic.trim();
        if topic.is_empty() {
            println!("{}", "Please provide a topic.".yellow());
            return Ok(Some(true));
        }

        let category = prompt_for_category()?;
        let mode = LearningMode::DeepDive {
            topic: topic.to_string(),
        };
        *session = Some(Session::new(mode, category));

        let prompt = build_deep_dive_prompt(topic);
        if let Some(ref mut sess) = session {
            send_and_display(&prompt, sess)?;
        }
        return Ok(Some(true));
    }

    if let Some(url) = input.strip_prefix("/link ") {
        let url = url.trim();
        if url.is_empty() {
            println!("{}", "Please provide a URL.".yellow());
            return Ok(Some(true));
        }

        let category = prompt_for_category()?;
        let mode = LearningMode::Link {
            url: url.to_string(),
        };
        *session = Some(Session::new(mode, category));

        let prompt = build_link_prompt(url);
        if let Some(ref mut sess) = session {
            send_and_display(&prompt, sess)?;
        }
        return Ok(Some(true));
    }

    if input.eq_ignore_ascii_case("/til") {
        if let Some(ref sess) = session {
            generate_and_save_til(sess, config)?;
        } else {
            println!(
                "{}",
                "No active session. Start with /learn or /link first.".yellow()
            );
        }
        return Ok(Some(true));
    }

    if input.eq_ignore_ascii_case("/note") {
        if let Some(ref sess) = session {
            generate_and_save_note(sess, config)?;
        } else {
            println!(
                "{}",
                "No active session. Start with /learn or /link first.".yellow()
            );
        }
        return Ok(Some(true));
    }

    Ok(None)
}

fn run_learning_session(mut session: Session, initial_prompt: String, config: &Config) -> Result<()> {
    println!("{}", "═".repeat(60).bright_cyan());
    println!(
        "{}",
        format!("  Learning: {}  ", session.topic())
            .bold()
            .bright_cyan()
    );
    println!("{}", "═".repeat(60).bright_cyan());
    println!();

    send_and_display(&initial_prompt, &mut session)?;

    println!();
    println!(
        "Commands: {} | {} | {}",
        "/til".green(),
        "/note".green(),
        "/exit".green()
    );
    println!();

    loop {
        let input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("holocron")
            .allow_empty(false)
            .interact_text()?;

        let input = input.trim();

        if input.eq_ignore_ascii_case("/quit") || input.eq_ignore_ascii_case("/exit") {
            println!("{}", "May the Force be with you.".bright_cyan());
            break;
        }

        if input.eq_ignore_ascii_case("/til") {
            generate_and_save_til(&session, config)?;
            continue;
        }

        if input.eq_ignore_ascii_case("/note") {
            generate_and_save_note(&session, config)?;
            continue;
        }

        send_and_display(input, &mut session)?;
    }

    Ok(())
}

fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    if let Ok(style) = ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}") {
        spinner.set_style(style);
    }
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    spinner
}

fn send_and_display(message: &str, session: &mut Session) -> Result<()> {
    let spinner = create_spinner("Consulting the archives...");

    let mut response = String::new();
    let mut first_chunk = true;

    let result = if let Some(ref session_id) = session.claude_session_id {
        claude::continue_conversation(session_id, message, |text| {
            if first_chunk {
                spinner.finish_and_clear();
                first_chunk = false;
            }
            print!("{}", text);
            io::stdout().flush().ok();
            response.push_str(text);
        })
    } else {
        let (resp, maybe_session_id) = claude::run_claude_command(message, |text| {
            if first_chunk {
                spinner.finish_and_clear();
                first_chunk = false;
            }
            print!("{}", text);
            io::stdout().flush().ok();
            response.push_str(text);
        })?;

        if let Some(sid) = maybe_session_id {
            session.set_session_id(sid);
        }
        Ok(resp)
    };

    if first_chunk {
        spinner.finish_and_clear();
    }

    println!();
    println!();

    match result {
        Ok(resp) => {
            session.add_exchange(message.to_string(), resp);
            Ok(())
        }
        Err(e) => {
            println!("{} {}", "Error:".red().bold(), e);
            Err(e)
        }
    }
}

fn prompt_for_category() -> Result<Option<String>> {
    let categories = vec![
        "git",
        "rust",
        "sql",
        "postgres",
        "python",
        "javascript",
        "Other (type custom)",
        "Skip (decide later)",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Category for TIL")
        .items(&categories)
        .default(0)
        .interact()?;

    match categories[selection] {
        "Skip (decide later)" => Ok(None),
        "Other (type custom)" => {
            let custom: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter category")
                .interact_text()?;
            Ok(Some(custom.to_lowercase()))
        }
        cat => Ok(Some(cat.to_string())),
    }
}

fn generate_and_save_til(session: &Session, config: &Config) -> Result<()> {
    println!();
    let spinner = create_spinner("Generating TIL...");

    let mut til_content = String::new();
    let mut first_chunk = true;

    til::generate_til(session, |text| {
        if first_chunk {
            spinner.finish_and_clear();
            println!("{}", "Generated TIL:".green().bold());
            println!("{}", "─".repeat(40));
            first_chunk = false;
        }
        print!("{}", text);
        io::stdout().flush().ok();
        til_content.push_str(text);
    })?;

    if first_chunk {
        spinner.finish_and_clear();
    }

    println!();
    println!("{}", "─".repeat(40));

    let title = til::writer::extract_title(&til_content).unwrap_or_else(|| "Untitled TIL".to_string());

    let category = session
        .category
        .clone()
        .map_or_else(prompt_category_input, Ok)?;

    let filename = til::writer::title_to_filename(&title);

    let confirm = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Save as {}/{}?", category, filename))
        .items(&["Yes, save it", "No, discard"])
        .default(0)
        .interact()?;

    if confirm == 0 {
        let path = til::write_til(&config.til_path, &config.archive_dir, &category, &filename, &til_content, &title)?;
        println!();
        println!("{} {}", "✓ TIL saved to:".green().bold(), path.display());
        println!("{}", "  README.md updated".dimmed());
    } else {
        println!("{}", "TIL discarded.".yellow());
    }

    Ok(())
}

fn generate_and_save_note(session: &Session, config: &Config) -> Result<()> {
    let notes_path = config.notes_path.as_ref().ok_or_else(|| {
        anyhow!("Notes path not configured. Run: holocron config --notes-path <path>")
    })?;

    println!();
    let spinner = create_spinner("Generating note...");

    let mut note_content = String::new();
    let mut first_chunk = true;

    notes::generate_note(session, |text| {
        if first_chunk {
            spinner.finish_and_clear();
            println!("{}", "Generated Note:".green().bold());
            println!("{}", "─".repeat(40));
            first_chunk = false;
        }
        print!("{}", text);
        io::stdout().flush().ok();
        note_content.push_str(text);
    })?;

    if first_chunk {
        spinner.finish_and_clear();
    }

    println!();
    println!("{}", "─".repeat(40));

    let title = notes::writer::extract_title(&note_content).unwrap_or_else(|| "Untitled Note".to_string());
    let filename = notes::writer::title_to_filename(&title);

    let confirm = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Save as {}?", filename))
        .items(&["Yes, save it", "No, discard"])
        .default(0)
        .interact()?;

    if confirm == 0 {
        let path = notes::write_note(notes_path, &filename, &note_content)?;
        println!();
        println!("{} {}", "✓ Note saved to:".green().bold(), path.display());
    } else {
        println!("{}", "Note discarded.".yellow());
    }

    Ok(())
}

fn prompt_category_input() -> Result<String> {
    let input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter category for this TIL")
        .interact_text()?;
    Ok(input.to_lowercase())
}
