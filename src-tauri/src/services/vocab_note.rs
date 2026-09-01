use crate::errors::AppError;
use crate::models::VocabNoteSnippet;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn resolve_note_path(path: &str) -> Result<PathBuf, AppError> {
    let trimmed = path
        .trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .trim();
    if trimmed.is_empty() {
        return Err(AppError::validation("Vocab note path is empty"));
    }

    let mut path = PathBuf::from(trimmed);
    if !path.is_absolute() {
        return Err(AppError::validation(
            "Vocab note path must be an absolute path",
        ));
    }

    let looks_like_dir = trimmed.ends_with(['/', '\\']) || path.is_dir() || path.extension().is_none();
    if looks_like_dir {
        path.push("vocab.md");
    }

    Ok(path)
}

pub fn format_vocab_note(
    word: &str,
    stars: i32,
    category: &str,
    snippets: &[VocabNoteSnippet],
) -> String {
    let star_label = if stars >= 2 { "★★" } else { "★" };
    let category_label = match category {
        "technical" => "技术",
        _ => "日常",
    };

    let mut out = format!("## {} · {} · {}\n", word, star_label, category_label);
    if snippets.is_empty() {
        out.push('\n');
        return out;
    }

    for snippet in snippets.iter().take(2) {
        let pos = snippet.part_of_speech.trim();
        if pos.is_empty() {
            out.push_str(&format!("- {}\n", snippet.definition.trim()));
        } else {
            out.push_str(&format!("- {}. {}\n", pos, snippet.definition.trim()));
        }
        if let Some(example) = snippet.example.as_deref() {
            let example = example.trim();
            if !example.is_empty() {
                out.push_str(&format!("  - {}\n", example));
            }
        }
    }
    out.push('\n');
    out
}

pub fn append_vocab_note(
    path: &Path,
    word: &str,
    stars: i32,
    category: &str,
    snippets: &[VocabNoteSnippet],
) -> Result<(), AppError> {
    ensure_note_file(path)?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(format_vocab_note(word, stars, category, snippets).as_bytes())?;
    file.flush()?;
    Ok(())
}

pub fn ensure_note_file(path: &Path) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    if path.exists() {
        return Ok(());
    }

    let mut file = OpenOptions::new().create(true).write(true).open(path)?;
    file.write_all(b"# Vocab\n\n")?;
    file.flush()?;
    log::info!("Created vocab note file at {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_two_senses_with_examples() {
        let note = format_vocab_note(
            "gain",
            1,
            "technical",
            &[
                VocabNoteSnippet {
                    part_of_speech: "n.".into(),
                    definition: "控制系统中输出与输入之比。".into(),
                    example: Some("The loop gain determines stability.".into()),
                },
                VocabNoteSnippet {
                    part_of_speech: "n.".into(),
                    definition: "获得；增加。".into(),
                    example: Some("She gained experience.".into()),
                },
            ],
        );

        assert!(note.starts_with("## gain · ★ · 技术\n"));
        assert!(note.contains("- n. 控制系统中输出与输入之比。"));
        assert!(note.contains("  - The loop gain determines stability."));
        assert!(note.contains("- n. 获得；增加。"));
    }

    #[test]
    fn rejects_relative_note_path() {
        let err = resolve_note_path("notes/vocab.md").unwrap_err();
        assert!(err.to_string().contains("absolute"));
    }

    #[test]
    fn directory_path_becomes_vocab_md() {
        let dir = tempfile::tempdir().unwrap();
        let resolved = resolve_note_path(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(resolved.file_name().unwrap(), "vocab.md");
    }

    #[test]
    fn creates_missing_folders_and_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("notes").join("vocab.md");
        assert!(!path.exists());

        append_vocab_note(
            &path,
            "gain",
            1,
            "daily",
            &[VocabNoteSnippet {
                part_of_speech: String::new(),
                definition: "to obtain".into(),
                example: None,
            }],
        )
        .unwrap();

        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.starts_with("# Vocab\n\n"));
        assert!(body.contains("## gain · ★ · 日常"));
        assert!(body.contains("- to obtain"));
    }

    #[test]
    fn ensure_note_file_creates_once() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing").join("vocab.md");
        ensure_note_file(&path).unwrap();
        ensure_note_file(&path).unwrap();
        let body = std::fs::read_to_string(&path).unwrap();
        assert_eq!(body, "# Vocab\n\n");
    }

    #[test]
    fn appends_to_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vocab.md");
        std::fs::write(&path, "# Vocab\n\n").unwrap();

        append_vocab_note(
            &path,
            "gain",
            2,
            "daily",
            &[VocabNoteSnippet {
                part_of_speech: String::new(),
                definition: "to obtain".into(),
                example: None,
            }],
        )
        .unwrap();

        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.starts_with("# Vocab\n\n"));
        assert!(body.contains("## gain · ★★ · 日常"));
        assert!(body.contains("- to obtain"));
    }
}
