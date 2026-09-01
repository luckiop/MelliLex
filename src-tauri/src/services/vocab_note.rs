use crate::errors::AppError;
use crate::models::VocabNoteSnippet;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

const MASTER_HEADER: &[u8] = b"# Vocab\n\n";
const SPLIT_FILES: [(&str, i32, &str); 4] = [
    ("1星-日常.md", 1, "daily"),
    ("2星-日常.md", 2, "daily"),
    ("1星-技术.md", 1, "technical"),
    ("2星-技术.md", 2, "technical"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteSyncKind {
    Skipped,
    Appended,
    Updated,
}

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

    let looks_like_dir =
        trimmed.ends_with(['/', '\\']) || path.is_dir() || path.extension().is_none();
    if looks_like_dir {
        path.push("vocab.md");
    }

    Ok(path)
}

pub fn split_note_dir(master: &Path) -> PathBuf {
    master
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(master)
        .to_path_buf()
}

pub fn split_file_name(stars: i32, category: &str) -> &'static str {
    match (stars >= 2, category) {
        (true, "technical") => "2星-技术.md",
        (true, _) => "2星-日常.md",
        (false, "technical") => "1星-技术.md",
        _ => "1星-日常.md",
    }
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
    file.write_all(MASTER_HEADER)?;
    file.flush()?;
    log::info!("Created vocab note file at {}", path.display());
    Ok(())
}

pub fn ensure_split_files(master: &Path) -> Result<(), AppError> {
    let dir = split_note_dir(master);
    for (name, _, _) in SPLIT_FILES {
        ensure_note_file(&dir.join(name))?;
    }
    Ok(())
}

pub fn format_master_section(
    word: &str,
    stars: i32,
    category: &str,
    snippets: &[VocabNoteSnippet],
) -> String {
    let pos = snippets
        .first()
        .map(|snippet| snippet.part_of_speech.trim())
        .filter(|pos| !pos.is_empty())
        .unwrap_or("-");
    let gloss = snippets
        .first()
        .map(heading_gloss)
        .filter(|gloss| !gloss.is_empty())
        .unwrap_or_default();
    let example = snippets
        .iter()
        .find_map(|snippet| snippet.example.as_deref())
        .map(str::trim)
        .filter(|example| !example.is_empty());
    let definition = snippets
        .first()
        .map(|snippet| snippet.definition.trim())
        .filter(|definition| !definition.is_empty());

    let mut out = format!("#### {} {}[{}]\n{}\n", word, pos, gloss, metadata_line(stars, category));
    if let Some(example) = example {
        out.push_str(example);
        out.push('\n');
    }
    if let Some(definition) = definition {
        out.push_str(definition);
        out.push('\n');
    }
    out.push('\n');
    out
}

const BRIEF_GLOSS_MAX_CJK: usize = 16;
const BRIEF_GLOSS_MAX_LATIN: usize = 32;
const FALLBACK_CJK_CHARS: usize = 8;
const FALLBACK_LATIN_WORDS: usize = 4;

pub(crate) fn heading_gloss(snippet: &VocabNoteSnippet) -> String {
    let gloss = snippet.brief_gloss.trim();
    let raw = if !gloss.is_empty() {
        gloss.to_string()
    } else {
        fallback_from_definition(&snippet.definition)
    };
    truncate_brief_gloss(&raw)
}

pub(crate) fn truncate_brief_gloss(text: &str) -> String {
    let trimmed = text.trim().replace('[', "").replace(']', "");
    let trimmed = trimmed.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let max = if trimmed.chars().any(is_cjk_char) {
        BRIEF_GLOSS_MAX_CJK
    } else {
        BRIEF_GLOSS_MAX_LATIN
    };
    if trimmed.chars().count() <= max {
        return trimmed.to_string();
    }
    trimmed.chars().take(max).collect()
}

fn fallback_from_definition(definition: &str) -> String {
    let trimmed = definition.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let cjk: String = trimmed.chars().filter(|c| is_cjk_char(*c)).take(FALLBACK_CJK_CHARS).collect();
    if !cjk.is_empty() {
        return cjk;
    }
    trimmed
        .split_whitespace()
        .take(FALLBACK_LATIN_WORDS)
        .collect::<Vec<_>>()
        .join(" ")
        .trim_end_matches(|c: char| matches!(c, '.' | ',' | ';' | ':' | '!' | '?'))
        .to_string()
}

fn is_cjk_char(c: char) -> bool {
    matches!(
        c,
        '\u{3400}'..='\u{4DBF}'
            | '\u{4E00}'..='\u{9FFF}'
            | '\u{F900}'..='\u{FAFF}'
            | '\u{20000}'..='\u{2A6DF}'
            | '\u{2A700}'..='\u{2B73F}'
            | '\u{2B740}'..='\u{2B81F}'
            | '\u{2B820}'..='\u{2CEAF}'
            | '\u{30000}'..='\u{323AF}'
    )
}

pub fn sync_vocab_notes(
    master: &Path,
    split_enabled: bool,
    word: &str,
    stars: i32,
    category: &str,
    snippets: &[VocabNoteSnippet],
) -> Result<NoteSyncKind, AppError> {
    ensure_note_file(master)?;
    let kind = sync_master_file(master, word, stars, category, snippets)?;
    if split_enabled {
        ensure_split_files(master)?;
        sync_split_files(master, word, stars, category)?;
    }
    Ok(kind)
}

pub fn remove_from_split_files(master: &Path, word: &str) -> Result<(), AppError> {
    let dir = split_note_dir(master);
    for (name, _, _) in SPLIT_FILES {
        let path = dir.join(name);
        if path.exists() {
            let mut notebook = read_notebook(&path)?;
            if notebook.remove_section(word) {
                write_notebook(&path, &notebook)?;
            }
        }
    }
    Ok(())
}

fn sync_master_file(
    master: &Path,
    word: &str,
    stars: i32,
    category: &str,
    snippets: &[VocabNoteSnippet],
) -> Result<NoteSyncKind, AppError> {
    let mut notebook = read_notebook(master)?;
    if notebook.has_h4_section(word) {
        if notebook.update_metadata(word, stars, category) {
            write_notebook(master, &notebook)?;
            return Ok(NoteSyncKind::Updated);
        }
        return Ok(NoteSyncKind::Skipped);
    }
    if notebook.has_legacy_heading(word) {
        return Ok(NoteSyncKind::Skipped);
    }
    notebook.append_section(format_master_section(word, stars, category, snippets));
    write_notebook(master, &notebook)?;
    Ok(NoteSyncKind::Appended)
}

fn sync_split_files(
    master: &Path,
    word: &str,
    stars: i32,
    category: &str,
) -> Result<(), AppError> {
    let dir = split_note_dir(master);
    let target_name = split_file_name(stars, category);
    let master_notebook = read_notebook(master)?;
    let Some(section) = master_notebook.section_text(word) else {
        return Ok(());
    };

    for (name, _, _) in SPLIT_FILES {
        let path = dir.join(name);
        ensure_note_file(&path)?;
        let mut notebook = read_notebook(&path)?;
        if name == target_name {
            if !notebook.has_h4_section(word) && !notebook.has_legacy_heading(word) {
                notebook.append_section(section.clone());
                write_notebook(&path, &notebook)?;
            }
        } else if notebook.remove_section(word) {
            write_notebook(&path, &notebook)?;
        }
    }
    Ok(())
}

fn metadata_line(stars: i32, category: &str) -> String {
    let stars = if stars >= 2 { "★★" } else { "★" };
    let category = if category == "technical" {
        "技术"
    } else {
        "日常"
    };
    format!("<small>{stars} · {category}</small>")
}

fn normalize_word_key(word: &str) -> String {
    word.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn is_h4_at(text: &str, index: usize) -> bool {
    if index > 0 && !text.is_char_boundary(index) {
        return false;
    }
    if index != 0 && !text[..index].ends_with('\n') {
        return false;
    }
    text[index..].starts_with("####")
}

fn h4_starts(text: &str) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut search_from = 0;
    while let Some(rel) = text[search_from..].find("####") {
        let abs = search_from + rel;
        if is_h4_at(text, abs) {
            starts.push(abs);
        }
        search_from = abs + 1;
        if search_from >= text.len() {
            break;
        }
    }
    starts
}

fn heading_word(heading: &str) -> Option<String> {
    let rest = heading.trim().strip_prefix("####")?.trim();
    let bracket = rest.rfind('[')?;
    let before = rest[..bracket].trim_end();
    let space = before.rfind(char::is_whitespace)?;
    let word = before[..space].trim();
    if word.is_empty() {
        None
    } else {
        Some(word.to_string())
    }
}

fn legacy_heading_word(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with("## ") || trimmed.starts_with("####") {
        return None;
    }
    let rest = trimmed.trim_start_matches('#').trim();
    let word = rest.split(" · ").next()?.trim();
    if word.is_empty() {
        None
    } else {
        Some(word.to_string())
    }
}

fn is_meta_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("<small>★") && trimmed.contains(" · ") && trimmed.ends_with("</small>")
}

fn split_first_line(text: &str) -> (&str, &str) {
    match text.find('\n') {
        Some(index) => {
            let line_end = if index > 0 && text.as_bytes()[index - 1] == b'\r' {
                index - 1
            } else {
                index
            };
            (&text[..line_end], &text[index + 1..])
        }
        None => (text, ""),
    }
}

fn newline_of(text: &str) -> &'static str {
    if text.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

struct Notebook {
    preamble: String,
    sections: Vec<String>,
}

impl Notebook {
    fn parse(text: &str) -> Self {
        let starts = h4_starts(text);
        if starts.is_empty() {
            return Self {
                preamble: text.to_string(),
                sections: Vec::new(),
            };
        }
        let preamble = text[..starts[0]].to_string();
        let mut sections = Vec::new();
        for (index, start) in starts.iter().enumerate() {
            let end = starts.get(index + 1).copied().unwrap_or(text.len());
            sections.push(text[*start..end].to_string());
        }
        Self { preamble, sections }
    }

    fn render(&self) -> String {
        let mut out = self.preamble.clone();
        for section in &self.sections {
            out.push_str(section);
        }
        out
    }

    fn has_h4_section(&self, word: &str) -> bool {
        self.section_index(word).is_some()
    }

    fn has_legacy_heading(&self, word: &str) -> bool {
        let key = normalize_word_key(word);
        self.preamble.lines().any(|line| {
            legacy_heading_word(line)
                .map(|found| normalize_word_key(&found) == key)
                .unwrap_or(false)
        })
    }

    fn section_index(&self, word: &str) -> Option<usize> {
        let key = normalize_word_key(word);
        self.sections.iter().position(|section| {
            let (heading, _) = split_first_line(section);
            heading_word(heading)
                .map(|found| normalize_word_key(&found) == key)
                .unwrap_or(false)
        })
    }

    fn section_text(&self, word: &str) -> Option<String> {
        self.section_index(word)
            .map(|index| self.sections[index].clone())
    }

    fn update_metadata(&mut self, word: &str, stars: i32, category: &str) -> bool {
        let Some(index) = self.section_index(word) else {
            return false;
        };
        let updated = apply_metadata(&self.sections[index], stars, category);
        if updated == self.sections[index] {
            return false;
        }
        self.sections[index] = updated;
        true
    }

    fn append_section(&mut self, section: String) {
        if !self.preamble.is_empty()
            && !self.preamble.ends_with('\n')
            && !section.is_empty()
        {
            self.preamble.push('\n');
        }
        self.sections.push(section);
    }

    fn remove_section(&mut self, word: &str) -> bool {
        if let Some(index) = self.section_index(word) {
            self.sections.remove(index);
            true
        } else {
            false
        }
    }
}

fn apply_metadata(section: &str, stars: i32, category: &str) -> String {
    let nl = newline_of(section);
    let desired = format!("{}{nl}", metadata_line(stars, category));
    let (heading, body) = split_first_line(section);
    let (next_line, rest) = split_first_line(body);
    if is_meta_line(next_line) {
        if rest.is_empty() {
            format!("{heading}{nl}{desired}")
        } else {
            format!("{heading}{nl}{desired}{rest}")
        }
    } else if body.is_empty() {
        format!("{heading}{nl}{desired}")
    } else {
        format!("{heading}{nl}{desired}{body}")
    }
}

fn read_notebook(path: &Path) -> Result<Notebook, AppError> {
    let text = if path.exists() {
        std::fs::read_to_string(path)?
    } else {
        String::from_utf8_lossy(MASTER_HEADER).into_owned()
    };
    Ok(Notebook::parse(&text))
}

fn write_notebook(path: &Path, notebook: &Notebook) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(path, notebook.render())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snippet(definition: &str, example: Option<&str>) -> VocabNoteSnippet {
        snippet_with_gloss("", definition, example)
    }

    fn snippet_with_gloss(
        brief_gloss: &str,
        definition: &str,
        example: Option<&str>,
    ) -> VocabNoteSnippet {
        VocabNoteSnippet {
            part_of_speech: "n.".into(),
            brief_gloss: brief_gloss.into(),
            definition: definition.into(),
            example: example.map(str::to_string),
        }
    }

    #[test]
    fn formats_heading_as_brief_gloss_and_keeps_full_definition() {
        let definition = "微控制器是一种将处理器、存储器和输入输出接口集成在单个芯片上的小型计算机，常用于嵌入式系统。";
        let note = format_master_section(
            "microcontroller",
            1,
            "technical",
            &[snippet_with_gloss(
                "微控制器",
                definition,
                Some("The microcontroller drives the motor."),
            )],
        );
        assert!(note.starts_with("#### microcontroller n.[微控制器]\n"));
        assert!(note.contains("<small>★ · 技术</small>"));
        assert!(note.contains("The microcontroller drives the motor.\n"));
        assert!(note.contains(definition));
        assert!(!note.contains("#### microcontroller n.[微控制器是一种"));
    }

    #[test]
    fn heading_prefers_brief_gloss_over_definition() {
        let note = format_master_section(
            "gain",
            2,
            "technical",
            &[snippet_with_gloss("增益", "放大器输出与输入之比。", None)],
        );
        assert!(note.starts_with("#### gain n.[增益]\n"));
        assert!(note.contains("放大器输出与输入之比。"));
    }

    #[test]
    fn heading_falls_back_to_first_cjk_chars_of_definition() {
        let definition = "微控制器是一种将处理器、存储器和输入输出接口集成在单个芯片上的小型计算机。";
        let note = format_master_section(
            "microcontroller",
            1,
            "technical",
            &[snippet(definition, None)],
        );
        assert!(note.starts_with("#### microcontroller n.[微控制器是一种将]\n"));
        assert!(note.contains(definition));
    }

    #[test]
    fn truncates_overflowing_brief_gloss() {
        let long_cjk = "一二三四五六七八九十十一十二十三";
        assert_eq!(truncate_brief_gloss(long_cjk).chars().count(), 16);
        let long_latin = "one two three four five six seven eight nine ten";
        assert_eq!(truncate_brief_gloss(long_latin).chars().count(), 32);
        assert_eq!(heading_gloss(&snippet_with_gloss("微控制器", "ignored", None)), "微控制器");
    }

    #[test]
    fn formats_heading_metadata_and_example() {
        let note = format_master_section(
            "dining table",
            2,
            "daily",
            &[snippet_with_gloss(
                "餐桌",
                "用餐的桌子",
                Some("They sat around the dining table."),
            )],
        );
        assert!(note.starts_with("#### dining table n.[餐桌]\n"));
        assert!(note.contains("<small>★★ · 日常</small>"));
        assert!(note.contains("They sat around the dining table."));
        assert!(note.contains("用餐的桌子"));
        assert!(!note.contains("## dining table"));
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
        let kind = sync_vocab_notes(
            &path,
            false,
            "gain",
            1,
            "daily",
            &[snippet("to obtain", Some("She gained experience."))],
        )
        .unwrap();
        assert_eq!(kind, NoteSyncKind::Appended);
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.starts_with("# Vocab\n\n"));
        assert!(body.contains("#### gain n.[to obtain]"));
        assert!(body.contains("<small>★ · 日常</small>"));
        assert!(body.contains("She gained experience."));
    }

    #[test]
    fn does_not_duplicate_existing_heading() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vocab.md");
        sync_vocab_notes(&path, false, "gain", 1, "daily", &[snippet("to obtain", None)]).unwrap();
        let kind = sync_vocab_notes(
            &path,
            false,
            "Gain",
            1,
            "daily",
            &[snippet("to obtain", None)],
        )
        .unwrap();
        assert_eq!(kind, NoteSyncKind::Skipped);
        let body = std::fs::read_to_string(&path).unwrap();
        assert_eq!(body.matches("#### gain").count(), 1);
    }

    #[test]
    fn star_change_replaces_metadata_and_keeps_user_notes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vocab.md");
        sync_vocab_notes(&path, false, "gain", 1, "daily", &[snippet("to obtain", None)]).unwrap();
        let mut body = std::fs::read_to_string(&path).unwrap();
        body.push_str("my personal note\n");
        std::fs::write(&path, body).unwrap();

        let kind = sync_vocab_notes(
            &path,
            false,
            "gain",
            2,
            "technical",
            &[snippet("loop gain", None)],
        )
        .unwrap();
        assert_eq!(kind, NoteSyncKind::Updated);
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("<small>★★ · 技术</small>"));
        assert!(body.contains("#### gain n.[to obtain]"));
        assert!(body.contains("my personal note"));
        assert!(!body.contains("loop gain"));
        assert_eq!(body.matches("#### gain").count(), 1);
    }

    #[test]
    fn skips_legacy_heading_without_rewriting() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vocab.md");
        std::fs::write(&path, "# Vocab\n\n## gain · ★ · 日常\n- to obtain\n").unwrap();
        let kind = sync_vocab_notes(
            &path,
            false,
            "gain",
            2,
            "daily",
            &[snippet("to obtain", None)],
        )
        .unwrap();
        assert_eq!(kind, NoteSyncKind::Skipped);
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("## gain · ★ · 日常"));
        assert!(!body.contains("#### gain"));
    }

    #[test]
    fn split_files_move_section_on_star_change() {
        let dir = tempfile::tempdir().unwrap();
        let master = dir.path().join("vocab.md");
        sync_vocab_notes(
            &master,
            true,
            "gain",
            1,
            "daily",
            &[snippet("to obtain", Some("She gained experience."))],
        )
        .unwrap();
        let daily_one = dir.path().join("1星-日常.md");
        let tech_two = dir.path().join("2星-技术.md");
        assert!(std::fs::read_to_string(&daily_one).unwrap().contains("#### gain"));

        let mut master_body = std::fs::read_to_string(&master).unwrap();
        master_body.push_str("keep me\n");
        std::fs::write(&master, master_body).unwrap();

        sync_vocab_notes(
            &master,
            true,
            "gain",
            2,
            "technical",
            &[snippet("loop", None)],
        )
        .unwrap();

        assert!(!std::fs::read_to_string(&daily_one).unwrap().contains("#### gain"));
        let moved = std::fs::read_to_string(&tech_two).unwrap();
        assert!(moved.contains("#### gain"));
        assert!(moved.contains("keep me"));
        assert!(moved.contains("<small>★★ · 技术</small>"));
    }

    #[test]
    fn delete_removes_split_entry_not_master() {
        let dir = tempfile::tempdir().unwrap();
        let master = dir.path().join("vocab.md");
        sync_vocab_notes(
            &master,
            true,
            "gain",
            1,
            "daily",
            &[snippet("to obtain", None)],
        )
        .unwrap();
        remove_from_split_files(&master, "gain").unwrap();
        assert!(std::fs::read_to_string(&master).unwrap().contains("#### gain"));
        assert!(!std::fs::read_to_string(dir.path().join("1星-日常.md"))
            .unwrap()
            .contains("#### gain"));
    }
}
