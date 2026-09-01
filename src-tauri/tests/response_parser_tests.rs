/// Integration tests for response parser
/// Tests parsing of AI responses using the public API
mod common;

use mellilex_lib::models::*;
use mellilex_lib::services::response_parser::ResponseParser;

#[test]
fn test_parse_partial_section1() {
    let response = r#"{
        "word": "test",
        "pronunciation": "/test/",
        "syllables": "test",
        "origin": "Latin testum",
        "formality": {
            "level": "Neutral",
            "percentage": 50
        },
        "tldr": "A procedure for evaluation"
    }"#;

    let result = ResponseParser::parse_partial::<WordSection1Header>(response, "section1");
    assert!(result.is_ok(), "Should parse section1 header");

    let section1 = result.unwrap();
    assert_eq!(section1.word, "test");
    assert_eq!(section1.pronunciation, "/test/");
}

#[test]
fn test_parse_partial_section2() {
    let response = r#"{
        "meanings": [{
            "number": 1,
            "partOfSpeech": "verb",
            "definition": "To examine or try out",
            "memoryTip": "Think of testing something out",
            "briefGloss": "检验",
            "examples": ["Test the water before diving"]
        }]
    }"#;

    let result = ResponseParser::parse_partial::<WordSection2Meanings>(response, "section2");
    assert!(result.is_ok(), "Should parse section2 meanings");

    let section2 = result.unwrap();
    assert_eq!(section2.meanings.len(), 1);
    assert_eq!(section2.meanings[0].part_of_speech, "verb");
    assert_eq!(section2.meanings[0].brief_gloss, "检验");
}

#[test]
fn test_parse_partial_section2_defaults_missing_brief_gloss() {
    let response = r#"{
        "meanings": [{
            "number": 1,
            "partOfSpeech": "noun",
            "definition": "A small computer on a chip",
            "memoryTip": "micro + controller",
            "examples": ["The MCU runs the loop"]
        }]
    }"#;

    let section2 = ResponseParser::parse_partial::<WordSection2Meanings>(response, "section2")
        .expect("Should parse section2 without briefGloss");
    assert_eq!(section2.meanings[0].brief_gloss, "");
}

#[test]
fn test_parse_partial_section3_related() {
    let response = r#"{
        "synonyms": ["examine", "try"],
        "antonyms": ["ignore", "skip"],
        "collocations": [
            {
                "phrase": "test out",
                "example": "Let's test out this theory"
            },
            {
                "phrase": "test drive",
                "example": "They test drove the car"
            }
        ]
    }"#;

    let result = ResponseParser::parse_partial::<WordSection3Related>(response, "section3_related");
    assert!(result.is_ok(), "Should parse section3 related words");

    let section3 = result.unwrap();
    assert_eq!(section3.synonyms.len(), 2);
    assert_eq!(section3.antonyms.len(), 2);
    assert_eq!(section3.collocations.len(), 2);
    assert_eq!(section3.collocations[0].phrase, "test out");
}
