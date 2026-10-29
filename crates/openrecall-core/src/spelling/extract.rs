#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtractedWord {
    pub canonical: String,
    pub display: String,
}
pub const EXTRACTION_POLICY_VERSION: u16 = 1;
fn punctuation(c: char) -> char {
    match c {
        '\u{2018}' | '\u{2019}' => '\'',
        '\u{2010}' | '\u{2011}' => '-',
        other => other,
    }
}
fn latin(c: char) -> bool {
    c.is_alphabetic() && c.script() == Script::Latin
}
fn letter_or_digit(c: char) -> bool {
    latin(c) || c.is_ascii_digit()
}

/// Policy 1 identity for a single complete term, never a substring or a phrase.
pub fn canonical_word(text: &str) -> Option<String> {
    let normalized: String = text
        .nfc()
        .map(punctuation)
        .flat_map(char::to_lowercase)
        .nfc()
        .collect();
    if matches!(normalized.as_str(), "c++" | "c#" | "f#") {
        return Some(normalized);
    }
    let mut chars = normalized.chars().peekable();
    let mut has_latin = false;
    let mut previous_part = false;
    let mut previous_latin = false;
    while let Some(c) = chars.next() {
        if latin(c) {
            has_latin = true;
            previous_part = true;
            previous_latin = true;
        } else if c.is_ascii_digit() {
            previous_part = true;
            previous_latin = false;
        } else if is_combining_mark(c) && previous_latin {
            // A combining mark belongs to its preceding Latin letter.
        } else if matches!(c, '\'' | '-')
            && previous_part
            && chars.peek().is_some_and(|c| letter_or_digit(*c))
        {
            previous_part = false;
            previous_latin = false;
        } else {
            return None;
        }
    }
    (has_latin && previous_part).then_some(normalized)
}

/// Unique policy-1 words in first appearance order. Link detection is local only.
pub fn extract_words(text: &str) -> Vec<ExtractedWord> {
    let mut finder = linkify::LinkFinder::new();
    finder.url_must_have_scheme(false);
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    for span in finder.spans(text).filter(|span| span.kind().is_none()) {
        let text = span.as_str();
        let mut words = text.unicode_word_indices().peekable();
        while let Some((start, first)) = words.next() {
            let mut end = start + first.len();
            while let Some(&(next_start, next)) = words.peek() {
                let gap = &text[end..next_start];
                let connector = gap.chars().count() == 1
                    && gap
                        .chars()
                        .all(|c| matches!(punctuation(c), '\'' | '-' | '_'));
                let symbol_candidate = matches!(&text[start..end], "C" | "c" | "F" | "f")
                    && !gap.is_empty()
                    && gap.chars().all(|c| matches!(c, '+' | '#'));
                if gap.is_empty() || connector || symbol_candidate {
                    end = next_start + next.len();
                    words.next();
                } else {
                    break;
                }
            }
            // Include the entire suffix, so C+++ is rejected rather than yielding C++.
            if matches!(&text[start..end], "C" | "c" | "F" | "f") {
                end += text[end..]
                    .chars()
                    .take_while(|c| matches!(c, '+' | '#'))
                    .map(char::len_utf8)
                    .sum::<usize>();
            }
            let candidate = &text[start..end];
            if let Some(canonical) = canonical_word(candidate)
                && seen.insert(canonical.clone())
            {
                let display = if canonical == "i" {
                    "I".into()
                } else {
                    candidate.nfc().collect()
                };
                result.push(ExtractedWord { canonical, display });
            }
        }
    }
    result
}
use std::collections::HashSet;
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};
use unicode_script::{Script, UnicodeScript};
use unicode_segmentation::UnicodeSegmentation;
