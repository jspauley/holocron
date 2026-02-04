use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

/// Stream message types from Claude CLI JSON output.
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum StreamMessage {
    /// System init message
    System {},
    /// Assistant response with full message
    Assistant { message: AssistantMessage },
    /// Final result with session_id
    Result {
        result: String,
        session_id: String,
    },
    /// Catch-all for other message types
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct AssistantMessage {
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ContentBlock {
    Text { text: String },
    #[serde(other)]
    Other,
}

/// Run a Claude command with the given prompt and stream the response
fn run_claude_with_args<F>(args: Vec<&str>, mut on_text: F) -> Result<(String, Option<String>)>
where
    F: FnMut(&str),
{
    let mut child = Command::new("claude")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("Failed to get stdout"))?;

    let reader = BufReader::new(stdout);
    let mut full_response = String::new();
    let mut session_id = None;

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        if let Ok(msg) = serde_json::from_str::<StreamMessage>(&line) {
            match msg {
                StreamMessage::Assistant { message } => {
                    // Extract text from content blocks
                    for block in message.content {
                        if let ContentBlock::Text { text } = block {
                            on_text(&text);
                            full_response.push_str(&text);
                        }
                    }
                }
                StreamMessage::Result {
                    result: _,
                    session_id: sid,
                } => {
                    session_id = Some(sid);
                }
                _ => {}
            }
        }
    }

    child.wait()?;
    Ok((full_response, session_id))
}

/// Run a single Claude command and return the full response
pub fn run_claude_command<F>(prompt: &str, on_text: F) -> Result<(String, Option<String>)>
where
    F: FnMut(&str),
{
    let args = vec![
        "--print",
        "--output-format",
        "stream-json",
        "--verbose",
        prompt,
    ];
    run_claude_with_args(args, on_text)
}

/// Continue a Claude conversation with an existing session
pub fn continue_conversation<F>(session_id: &str, message: &str, on_text: F) -> Result<String>
where
    F: FnMut(&str),
{
    let args = vec![
        "--print",
        "--output-format",
        "stream-json",
        "--verbose",
        "--resume",
        session_id,
        message,
    ];
    let (response, _) = run_claude_with_args(args, on_text)?;
    Ok(response)
}
