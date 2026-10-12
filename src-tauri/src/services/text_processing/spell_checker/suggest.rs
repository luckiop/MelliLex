use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

const MIN_QUERY_CHARS: usize = 2;
const MAX_QUERY_CHARS: usize = 32;
/// Ignore dictionary hapaxes. The bundled list is already above this.
const MIN_PREFIX_COUNT: i64 = 1_000;

#[derive(Debug, Clone)]
pub struct DictEntry {
    pub word: String,
    pub count: i64,
}

#[derive(Debug, Clone)]
pub struct FuzzyHit {
    pub term: String,
    pub distance: i64,
    pub count: i64,
}

pub fn normalize_query(raw: &str) -> Option<String> {
    let query = raw.trim().to_lowercase();
    let len = query.chars().count();
    if !(MIN_QUERY_CHARS..=MAX_QUERY_CHARS).contains(&len) {
        return None;
    }
    let letters = query
        .chars()
        .all(|c| c.is_ascii_alphabetic() || c == '\'' || c == '-');
    let has_letter = query.chars().any(|c| c.is_ascii_alphabetic());
    if letters && has_letter {
        Some(query)
    } else {
        None
    }
}

pub fn load_entries(text: &str) -> Vec<DictEntry> {
    let mut best: HashMap<String, i64> = HashMap::new();
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let Some(word) = parts.next() else { continue };
        let Some(count) = parts.next().and_then(|value| value.parse::<i64>().ok()) else {
            continue;
        };
        let Some(word) = normalize_query(word) else { continue };
        let slot = best.entry(word).or_insert(0);
        if count > *slot {
            *slot = count;
        }
    }
    let mut entries: Vec<DictEntry> = best
        .into_iter()
        .map(|(word, count)| DictEntry { word, count })
        .collect();
    entries.sort_by(|a, b| a.word.cmp(&b.word));
    entries
}

fn prefix_bounds(entries: &[DictEntry], prefix: &str) -> (usize, usize) {
    let start = entries.partition_point(|entry| entry.word.as_str() < prefix);
    let end = start
        + entries[start..]
            .partition_point(|entry| entry.word.starts_with(prefix));
    (start, end)
}

fn top_prefix(entries: &[DictEntry], prefix: &str, limit: usize) -> Vec<DictEntry> {
    if limit == 0 {
        return Vec::new();
    }
    let (start, end) = prefix_bounds(entries, prefix);
    let mut heap: BinaryHeap<Reverse<(i64, usize)>> = BinaryHeap::new();
    for (offset, entry) in entries[start..end].iter().enumerate() {
        if entry.count < MIN_PREFIX_COUNT {
            continue;
        }
        let index = start + offset;
        if heap.len() < limit {
            heap.push(Reverse((entry.count, index)));
        } else if let Some(Reverse((worst, _))) = heap.peek() {
            if entry.count > *worst {
                heap.pop();
                heap.push(Reverse((entry.count, index)));
            }
        }
    }
    let mut ranked: Vec<DictEntry> = heap
        .into_iter()
        .map(|Reverse((_, index))| entries[index].clone())
        .collect();
    ranked.sort_by(|a, b| b.count.cmp(&a.count).then(a.word.cmp(&b.word)));
    ranked
}

/// Prefix completions first, then typo corrections such as captian -> captain.
pub fn suggest_from(
    entries: &[DictEntry],
    fuzzy: &[FuzzyHit],
    query: &str,
    limit: usize,
) -> Vec<String> {
    let Some(query) = normalize_query(query) else {
        return Vec::new();
    };
    if limit == 0 {
        return Vec::new();
    }

    let prefix = top_prefix(entries, &query, limit);
    let mut corrections: Vec<&FuzzyHit> = fuzzy
        .iter()
        .filter(|hit| {
            hit.distance > 0
                && hit.term != query
                && !hit.term.starts_with(&query)
                && !query.starts_with(&hit.term)
        })
        .collect();
    corrections.sort_by(|a, b| a.distance.cmp(&b.distance).then(b.count.cmp(&a.count)));

    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let mut push = |term: &str| {
        if out.len() >= limit || term.is_empty() || !seen.insert(term.to_string()) {
            return;
        }
        out.push(term.to_string());
    };

    // A real prefix ("capt") should stay a completion list.
    // A typo with no common continuation ("captian") leads with the correction.
    if prefix.is_empty() {
        for hit in &corrections {
            push(&hit.term);
        }
    }
    for entry in &prefix {
        push(&entry.word);
    }
    if !prefix.is_empty() {
        for hit in &corrections {
            push(&hit.term);
        }
    }
    out.truncate(limit);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(word: &str, count: i64) -> DictEntry {
        DictEntry {
            word: word.to_string(),
            count,
        }
    }

    fn hit(term: &str, distance: i64, count: i64) -> FuzzyHit {
        FuzzyHit {
            term: term.to_string(),
            distance,
            count,
        }
    }

    #[test]
    fn prefix_completions_rank_by_frequency() {
        let mut entries = vec![
            entry("captain", 50_000),
            entry("caption", 20_000),
            entry("captive", 5_000),
            entry("card", 9_000_000),
        ];
        entries.sort_by(|a, b| a.word.cmp(&b.word));
        let words = suggest_from(&entries, &[], "capt", 8);
        assert_eq!(words, vec!["captain", "caption", "captive"]);
    }

    #[test]
    fn transposed_typo_leads_when_nothing_continues_the_prefix() {
        let mut entries = vec![entry("captain", 50_000), entry("caption", 20_000)];
        entries.sort_by(|a, b| a.word.cmp(&b.word));
        let fuzzy = vec![hit("captain", 1, 50_000), hit("caption", 2, 20_000)];
        let words = suggest_from(&entries, &fuzzy, "captian", 8);
        assert_eq!(words.first().map(String::as_str), Some("captain"));
    }

    #[test]
    fn rejects_phrases_and_single_letters() {
        assert!(normalize_query("c").is_none());
        assert!(normalize_query("break the ice").is_none());
        assert!(normalize_query("餐桌").is_none());
        assert_eq!(normalize_query("Capt"), Some("capt".to_string()));
    }
}
