use crate::errors::AppError;

const MAX_WORD_LENGTH: usize = 100;

const MAX_TRANSLATION_LENGTH: usize = 8000;

const MAX_CONTEXT_LENGTH: usize = 1000;

const MAX_MARKDOWN_LENGTH: usize = 1_000_000; // 1MB

const MAX_PRACTICE_EXERCISES: usize = 20;

pub fn validate_word_query(word: &str) -> Result<String, AppError> {
    let trimmed = word.trim();

    if trimmed.is_empty() {
        return Err(AppError::validation("Word query cannot be empty"));
    }

    if trimmed.len() > MAX_WORD_LENGTH {
        return Err(AppError::validation(format!(
            "Word query too long (max {} characters)",
            MAX_WORD_LENGTH
        )));
    }

    // Remove leading/trailing non-alphanumeric chars but keep internal ones (e.g., "mother-in-law")
    let sanitized = trimmed
        .trim_start_matches(|c: char| !c.is_alphanumeric())
        .trim_end_matches(|c: char| !c.is_alphanumeric());

    if sanitized.is_empty() {
        return Err(AppError::validation(
            "Word query contains only special characters",
        ));
    }

    Ok(sanitized.to_string())
}

pub fn validate_translation_query(source: &str) -> Result<String, AppError> {
    let trimmed = source.trim();

    if trimmed.is_empty() {
        return Err(AppError::validation("Translation query cannot be empty"));
    }

    if trimmed.chars().count() > MAX_TRANSLATION_LENGTH {
        return Err(AppError::validation(format!(
            "Translation query too long (max {} characters)",
            MAX_TRANSLATION_LENGTH
        )));
    }

    Ok(trimmed.to_string())
}

pub fn validate_context(context: &str) -> Result<String, AppError> {
    let trimmed = context.trim();

    if trimmed.is_empty() {
        return Err(AppError::validation("Context cannot be empty"));
    }

    if trimmed.len() > MAX_CONTEXT_LENGTH {
        return Err(AppError::validation(format!(
            "Context too long (max {} characters)",
            MAX_CONTEXT_LENGTH
        )));
    }

    Ok(trimmed.to_string())
}

pub fn validate_exercise_count(count: usize) -> Result<usize, AppError> {
    if count == 0 {
        return Err(AppError::validation("Exercise count must be at least 1"));
    }

    if count > MAX_PRACTICE_EXERCISES {
        return Err(AppError::validation(format!(
            "Exercise count too high (max {})",
            MAX_PRACTICE_EXERCISES
        )));
    }

    Ok(count)
}

pub fn validate_markdown(markdown: &str) -> Result<&str, AppError> {
    if markdown.is_empty() {
        return Err(AppError::validation("Markdown content cannot be empty"));
    }

    if markdown.len() > MAX_MARKDOWN_LENGTH {
        return Err(AppError::validation(format!(
            "Markdown content too large (max {} bytes)",
            MAX_MARKDOWN_LENGTH
        )));
    }

    Ok(markdown)
}

pub fn validate_api_token(token: &str) -> Result<&str, AppError> {
    let trimmed = token.trim();

    if trimmed.is_empty() {
        return Err(AppError::validation("API token cannot be empty"));
    }

    // Most API tokens are at least 20 characters
    if trimmed.len() < 10 {
        return Err(AppError::validation(
            "API token appears too short (min 10 characters)",
        ));
    }

    Ok(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_word_query_success() {
        assert_eq!(validate_word_query("  hello  ").unwrap(), "hello");
        assert_eq!(
            validate_word_query("mother-in-law").unwrap(),
            "mother-in-law"
        );
        assert_eq!(validate_word_query("'hello'").unwrap(), "hello");
        assert_eq!(validate_word_query("  !test?  ").unwrap(), "test");
    }

    #[test]
    fn test_validate_word_query_failures() {
        assert!(validate_word_query("").is_err());
        assert!(validate_word_query("   ").is_err());
        assert!(validate_word_query("!!!").is_err());
        assert!(validate_word_query(&"a".repeat(MAX_WORD_LENGTH + 1)).is_err());
    }

    #[test]
    fn test_validate_translation_query() {
        let long = "Bai Yu (白宇) is a lecturer in the College of Education Sciences.";
        assert_eq!(validate_translation_query(&format!("  {long}  ")).unwrap(), long);
        assert!(validate_translation_query("").is_err());
        assert!(validate_translation_query(&"字".repeat(MAX_TRANSLATION_LENGTH + 1)).is_err());
        assert!(validate_translation_query(&"字".repeat(MAX_TRANSLATION_LENGTH)).is_ok());
    }

    #[test]
    fn test_validate_context_success() {
        assert_eq!(
            validate_context("  example context  ").unwrap(),
            "example context"
        );
    }

    #[test]
    fn test_validate_context_failures() {
        assert!(validate_context("").is_err());
        assert!(validate_context("   ").is_err());
        assert!(validate_context(&"a".repeat(MAX_CONTEXT_LENGTH + 1)).is_err());
    }

    #[test]
    fn test_validate_exercise_count() {
        assert_eq!(validate_exercise_count(5).unwrap(), 5);
        assert!(validate_exercise_count(0).is_err());
        assert!(validate_exercise_count(MAX_PRACTICE_EXERCISES + 1).is_err());
    }

    #[test]
    fn test_validate_api_token() {
        assert_eq!(
            validate_api_token("  sk-1234567890  ").unwrap(),
            "sk-1234567890"
        );
        assert!(validate_api_token("").is_err());
        assert!(validate_api_token("short").is_err());
    }
}
