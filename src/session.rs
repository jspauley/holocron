use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum LearningMode {
    DeepDive { topic: String },
    Link { url: String },
}

impl fmt::Display for LearningMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LearningMode::DeepDive { topic } => write!(f, "Deep Dive: {}", topic),
            LearningMode::Link { url } => write!(f, "Link Analysis: {}", url),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Exchange {
    pub user_message: String,
    pub assistant_response: String,
}

#[derive(Debug)]
pub struct Session {
    pub mode: LearningMode,
    pub category: Option<String>,
    pub exchanges: Vec<Exchange>,
    pub claude_session_id: Option<String>,
}

impl Session {
    pub fn new(mode: LearningMode, category: Option<String>) -> Self {
        Self {
            mode,
            category,
            exchanges: Vec::new(),
            claude_session_id: None,
        }
    }

    pub fn add_exchange(&mut self, user_message: String, assistant_response: String) {
        self.exchanges.push(Exchange {
            user_message,
            assistant_response,
        });
    }

    pub fn set_session_id(&mut self, session_id: String) {
        self.claude_session_id = Some(session_id);
    }

    /// Build context summary for TIL generation
    pub fn build_til_context(&self) -> String {
        let mut context = String::new();

        context.push_str(&format!("Learning Session: {}\n\n", self.mode));

        if let Some(ref cat) = self.category {
            context.push_str(&format!("Category: {}\n\n", cat));
        }

        context.push_str("Conversation Summary:\n");
        for (i, exchange) in self.exchanges.iter().enumerate() {
            context.push_str(&format!("\n--- Exchange {} ---\n", i + 1));
            context.push_str(&format!("User: {}\n", exchange.user_message));
            context.push_str(&format!(
                "Assistant: {}\n",
                truncate_for_context(&exchange.assistant_response, 500)
            ));
        }

        context
    }

    /// Get the main topic/subject of this session
    pub fn topic(&self) -> &str {
        match &self.mode {
            LearningMode::DeepDive { topic } => topic,
            LearningMode::Link { url } => url,
        }
    }
}

fn truncate_for_context(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learning_mode_display_deep_dive() {
        let mode = LearningMode::DeepDive {
            topic: "Rust ownership".to_string(),
        };
        assert_eq!(format!("{}", mode), "Deep Dive: Rust ownership");
    }

    #[test]
    fn test_learning_mode_display_link() {
        let mode = LearningMode::Link {
            url: "https://example.com".to_string(),
        };
        assert_eq!(format!("{}", mode), "Link Analysis: https://example.com");
    }

    #[test]
    fn test_session_new() {
        let mode = LearningMode::DeepDive {
            topic: "test".to_string(),
        };
        let session = Session::new(mode.clone(), Some("rust".to_string()));

        assert_eq!(session.mode, mode);
        assert_eq!(session.category, Some("rust".to_string()));
        assert!(session.exchanges.is_empty());
        assert!(session.claude_session_id.is_none());
    }

    #[test]
    fn test_session_add_exchange() {
        let mode = LearningMode::DeepDive {
            topic: "test".to_string(),
        };
        let mut session = Session::new(mode, None);

        session.add_exchange("Hello".to_string(), "Hi there".to_string());

        assert_eq!(session.exchanges.len(), 1);
        assert_eq!(session.exchanges[0].user_message, "Hello");
        assert_eq!(session.exchanges[0].assistant_response, "Hi there");
    }

    #[test]
    fn test_session_set_session_id() {
        let mode = LearningMode::DeepDive {
            topic: "test".to_string(),
        };
        let mut session = Session::new(mode, None);

        session.set_session_id("abc123".to_string());

        assert_eq!(session.claude_session_id, Some("abc123".to_string()));
    }

    #[test]
    fn test_session_topic_deep_dive() {
        let mode = LearningMode::DeepDive {
            topic: "Rust".to_string(),
        };
        let session = Session::new(mode, None);

        assert_eq!(session.topic(), "Rust");
    }

    #[test]
    fn test_session_topic_link() {
        let mode = LearningMode::Link {
            url: "https://example.com".to_string(),
        };
        let session = Session::new(mode, None);

        assert_eq!(session.topic(), "https://example.com");
    }

    #[test]
    fn test_build_til_context_with_category() {
        let mode = LearningMode::DeepDive {
            topic: "Git".to_string(),
        };
        let mut session = Session::new(mode, Some("git".to_string()));
        session.add_exchange("How does rebase work?".to_string(), "Rebase replays commits...".to_string());

        let context = session.build_til_context();

        assert!(context.contains("Deep Dive: Git"));
        assert!(context.contains("Category: git"));
        assert!(context.contains("How does rebase work?"));
        assert!(context.contains("Rebase replays commits..."));
    }

    #[test]
    fn test_build_til_context_without_category() {
        let mode = LearningMode::Link {
            url: "https://example.com".to_string(),
        };
        let session = Session::new(mode, None);

        let context = session.build_til_context();

        assert!(context.contains("Link Analysis: https://example.com"));
        assert!(!context.contains("Category:"));
    }

    #[test]
    fn test_truncate_for_context_short() {
        let result = truncate_for_context("short", 100);
        assert_eq!(result, "short");
    }

    #[test]
    fn test_truncate_for_context_long() {
        let long_text = "a".repeat(600);
        let result = truncate_for_context(&long_text, 500);

        assert_eq!(result.len(), 503); // 500 + "..."
        assert!(result.ends_with("..."));
    }
}
