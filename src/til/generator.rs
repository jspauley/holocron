use crate::claude::{continue_conversation, run_claude_command};
use crate::session::Session;
use anyhow::Result;

/// Generate a TIL from the current session using the /til skill
pub fn generate_til<F>(session: &Session, on_text: F) -> Result<String>
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
        r#"Based on our learning session, generate a TIL (Today I Learned) entry.

{}

Use /til to generate the markdown content. The TIL should capture the most important, actionable learning from this session - something someone could quickly reference later.

Focus on the practical "how to" aspect with working code examples."#,
        context
    )
}
