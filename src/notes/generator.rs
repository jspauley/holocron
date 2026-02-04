use crate::claude::{continue_conversation, run_claude_command};
use crate::session::Session;
use anyhow::Result;

/// Generate a comprehensive note from the current session using the /note skill
pub fn generate_note<F>(session: &Session, on_text: F) -> Result<String>
where
    F: FnMut(&str),
{
    let prompt = build_generation_prompt(session);

    // If we have an existing session, continue it to maintain context
    if let Some(ref session_id) = session.claude_session_id {
        continue_conversation(session_id, &prompt, on_text)
    } else {
        // Start fresh with full context
        let (response, _) = run_claude_command(&prompt, on_text)?;
        Ok(response)
    }
}

fn build_generation_prompt(session: &Session) -> String {
    let context = session.build_til_context();

    format!(
        r#"Based on our learning session, generate a comprehensive knowledge base note.

{}

Use /note to generate the markdown content. The note should be thorough and detailed - this is for a personal knowledge base, not a quick reference.

Include:
- YAML frontmatter with title, date, tags, and aliases
- Detailed explanations of concepts
- Code examples with annotations
- Key insights from our Q&A
- Related topics as wiki-links"#,
        context
    )
}
