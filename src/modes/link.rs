/// Build the initial prompt for analyzing a link/article
pub fn build_link_prompt(url: &str) -> String {
    format!(
        r#"Please analyze this article/resource: {}

Provide:
1. A brief summary of the main points
2. Key technical concepts explained
3. Practical takeaways or code examples if applicable
4. Your assessment of what's most valuable to learn from this

Use WebFetch to access the content, then explain it thoroughly. I'll ask follow-up questions about specific parts."#,
        url
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_link_prompt_contains_url() {
        let prompt = build_link_prompt("https://example.com/article");
        assert!(prompt.contains("https://example.com/article"));
    }

    #[test]
    fn test_build_link_prompt_contains_sections() {
        let prompt = build_link_prompt("https://test.com");
        assert!(prompt.contains("brief summary"));
        assert!(prompt.contains("Key technical concepts"));
        assert!(prompt.contains("Practical takeaways"));
        assert!(prompt.contains("WebFetch"));
    }
}
