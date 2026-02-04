/// Build the initial prompt for a deep dive learning session
pub fn build_deep_dive_prompt(topic: &str) -> String {
    format!(
        r#"I want to learn about: {}

Please explain this topic in technical detail. Cover:
1. Core concepts and how they work
2. Practical examples with code where applicable
3. Common use cases and best practices
4. Common pitfalls to avoid

Be thorough but focused. I'll ask follow-up questions to go deeper on specific aspects."#,
        topic
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_deep_dive_prompt_contains_topic() {
        let prompt = build_deep_dive_prompt("Rust ownership");
        assert!(prompt.contains("Rust ownership"));
    }

    #[test]
    fn test_build_deep_dive_prompt_contains_sections() {
        let prompt = build_deep_dive_prompt("test");
        assert!(prompt.contains("Core concepts"));
        assert!(prompt.contains("Practical examples"));
        assert!(prompt.contains("Common use cases"));
        assert!(prompt.contains("Common pitfalls"));
    }
}
