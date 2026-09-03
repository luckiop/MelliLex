use crate::services::response_processing::{JsonRepairer, ResponseSanitizer};
use anyhow::Result;
use serde::de::DeserializeOwned;

pub struct ResponseParser;

impl ResponseParser {
    pub fn parse_partial<T>(response: &str, context: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        log::debug!("Parsing {} response", context);

        let sanitized = ResponseSanitizer::sanitize(response);
        let sanitized_str = sanitized.as_ref();

        Self::parse_with_repair(sanitized_str, context).map_err(|e| {
            let preview = Self::truncate_for_log(sanitized_str, 500);
            log::error!(
                "Failed to parse {} JSON: {}\nResponse preview: {}",
                context,
                e,
                preview
            );
            anyhow::anyhow!("JSON parse error in {}: {}", context, e)
        })
    }

    fn truncate_for_log(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}... [truncated, {} total chars]", &s[..max_len], s.len())
        }
    }

    #[cfg(test)]
    pub fn parse_field<T>(response: &str, field: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        log::debug!("Parsing field '{}' from response", field);

        let sanitized = ResponseSanitizer::sanitize(response);
        let sanitized_str = sanitized.as_ref();

        let value: serde_json::Value = serde_json::from_str(sanitized_str).map_err(|e| {
            log::error!(
                "Failed to parse JSON while extracting field '{}': {}",
                field,
                e
            );
            log::debug!("Sanitized response: {}", sanitized_str);
            log::debug!("Response: {}", response);
            anyhow::anyhow!("JSON parse error while extracting '{}': {}", field, e)
        })?;

        let field_value = value.get(field).ok_or_else(|| {
            log::error!("Field '{}' missing in response", field);
            anyhow::anyhow!("Field '{}' not found in response", field)
        })?;

        serde_json::from_value(field_value.clone()).map_err(|e| {
            log::error!("Failed to deserialize field '{}' value: {}", field, e);
            anyhow::anyhow!("JSON parse error in field '{}': {}", field, e)
        })
    }

    /// Parses JSON with automatic repair for truncated responses.
    /// Attempts repair if initial parse fails with EOF error.
    fn parse_with_repair<T>(
        sanitized: &str,
        context: &str,
    ) -> std::result::Result<T, serde_json::Error>
    where
        T: DeserializeOwned,
    {
        match serde_json::from_str::<T>(sanitized) {
            Ok(value) => Ok(value),
            Err(err) => {
                let syntax = JsonRepairer::repair_syntax(sanitized);
                if syntax != sanitized {
                    if let Ok(value) = serde_json::from_str::<T>(&syntax) {
                        log::warn!("{} JSON repaired (trailing/missing commas)", context);
                        return Ok(value);
                    }
                    if let Some(repair) = JsonRepairer::repair_truncated_json(&syntax) {
                        if let Ok(value) = serde_json::from_str::<T>(&repair.text) {
                            log::warn!(
                                "{} JSON repaired (syntax + {} closers)",
                                context,
                                repair.appended
                            );
                            return Ok(value);
                        }
                    }
                }

                if err.classify() == serde_json::error::Category::Eof {
                    if let Some(repair) = JsonRepairer::repair_truncated_json(sanitized) {
                        log::warn!(
                            "{} JSON appeared truncated. Appending {} closing delimiters to repair",
                            context,
                            repair.appended
                        );
                        return serde_json::from_str::<T>(&repair.text).map_err(|repair_err| {
                            log::error!(
                                "{} JSON still invalid after repair attempt: {}",
                                context,
                                repair_err
                            );
                            repair_err
                        });
                    }
                }
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn parse_partial_success() {
        #[derive(serde::Deserialize)]
        struct TestData {
            value: String,
        }

        let response = r#"{"value": "test"}"#;
        let result: Result<TestData> = ResponseParser::parse_partial(response, "test");
        let parsed = result.expect("parse_partial should succeed");
        assert_eq!(parsed.value, "test");
    }

    #[test]
    fn parse_partial_with_code_fence() {
        #[derive(serde::Deserialize)]
        struct TestData {
            value: String,
        }

        let response = "```json\n{\n  \"value\": \"wrapped\"\n}\n```";
        let result: Result<TestData> = ResponseParser::parse_partial(response, "test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, "wrapped");
    }

    #[test]
    fn parse_field_with_code_fence() {
        let response = "```json\n{\n  \"value\": 42\n}\n```";
        let result: Result<i32> = ResponseParser::parse_field(response, "value");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn parse_partial_with_uppercase_language_tag() {
        #[derive(serde::Deserialize)]
        struct TestData {
            value: u32,
        }

        let response = "```JSON\r\n{\n  \"value\": 7\n}\n```";
        let result: Result<TestData> = ResponseParser::parse_partial(response, "test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, 7);
    }

    #[test]
    fn parse_partial_with_surrounding_text() {
        #[derive(serde::Deserialize)]
        struct TestData {
            value: u32,
        }

        let response = "Here is the payload:\n{\n  \"value\": 99\n}\nThanks!";
        let result: Result<TestData> = ResponseParser::parse_partial(response, "test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, 99);
    }

    #[test]
    fn parse_partial_with_array_segment() {
        let response = "Some data: [1, 2, 3] end";
        let result: Result<Vec<i32>> = ResponseParser::parse_partial(response, "array");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn parse_field_invalid_json_reports_context() {
        let error = ResponseParser::parse_field::<Vec<String>>("not json", "meanings")
            .expect_err("invalid JSON should produce an error");
        let message = error.to_string();
        assert!(
            message.contains("meanings"),
            "Error message should reference the requested field"
        );
    }

    #[test]
    fn parse_partial_repairs_truncated_json() {
        #[derive(Deserialize)]
        struct DomainWrapper {
            domain_explorations: Vec<DomainEntry>,
        }

        #[derive(Deserialize)]
        struct DomainEntry {
            domain: String,
        }

        let truncated = r#"{
  "domain_explorations": [
    {
      "domain": "Academic"
    }
  "#;

        let parsed: Result<DomainWrapper> =
            ResponseParser::parse_partial(truncated, "domain exploration");
        let parsed = parsed.expect("Parser should repair truncated JSON");
        assert_eq!(parsed.domain_explorations.len(), 1);
        assert_eq!(parsed.domain_explorations[0].domain, "Academic");
    }

    #[test]
    fn parse_partial_repairs_missing_comma_between_meanings() {
        #[derive(Deserialize)]
        struct Wrapper {
            meanings: Vec<Meaning>,
        }
        #[derive(Deserialize)]
        struct Meaning {
            #[allow(dead_code)]
            number: i32,
        }

        let input = r#"{
  "meanings": [
    { "number": 1 }
    { "number": 2 }
  ]
}"#;
        let parsed: Wrapper =
            ResponseParser::parse_partial(input, "section2_meanings").expect("repair missing comma");
        assert_eq!(parsed.meanings.len(), 2);
    }
}
