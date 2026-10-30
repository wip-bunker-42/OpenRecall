use openrecall_core::spelling::extract::{canonical_word, extract_words};

fn keys(text: &str) -> Vec<String> {
    extract_words(text)
        .into_iter()
        .map(|w| w.canonical)
        .collect()
}

#[test]
fn unknown_words_contractions_and_compounds_are_whole() {
    assert_eq!(
        keys("don't don’t well-known OpenAI B2B GPT-4 C++ C# F#"),
        [
            "don't",
            "well-known",
            "openai",
            "b2b",
            "gpt-4",
            "c++",
            "c#",
            "f#"
        ]
    );
    assert_eq!(
        keys("‘Quorvex’ John's well‐known well‑known red—blue green–yellow"),
        [
            "quorvex",
            "john's",
            "well-known",
            "red",
            "blue",
            "green",
            "yellow"
        ]
    );
    let words = extract_words("HELLO hello i I John's john’s");
    assert_eq!(
        words.iter().map(|w| w.display.as_str()).collect::<Vec<_>>(),
        ["HELLO", "I", "John's"]
    );
}

#[test]
fn links_numbers_and_mixed_scripts_do_not_leak_fragments() {
    assert_eq!(
        keys("a I the is 2026 test@example.com https://example.com"),
        ["a", "i", "the", "is"]
    );
    assert_eq!(
        keys(
            "قبل English بعد example.org/path www.example.com (https://example.net/a?q=word) user@example.org final"
        ),
        ["english", "final"]
    );
    assert_eq!(
        keys(
            "helloالعربية abc漢字 Ελληνικά中文 clean snake_case 123_foo 12.34 ٢٠٢٦ e.g. Node.js end"
        ),
        ["clean", "end"]
    );
    for invalid in [
        "123",
        "snake_case",
        "helloالعربية",
        "abc漢字",
        "C++Builder",
        "F##",
        "-bad",
        "bad-",
        "'bad'",
        "two words",
        "",
    ] {
        assert_eq!(canonical_word(invalid), None, "{invalid}");
    }
}

#[test]
fn canonical_keys_preserve_distinct_inflections() {
    assert_eq!(
        keys("Learn learn learned learning café cafe\u{301} CAFE résumé resume"),
        [
            "learn", "learned", "learning", "café", "cafe", "résumé", "resume"
        ]
    );
    assert_eq!(canonical_word("DON’T"), Some("don't".into()));
    assert_eq!(canonical_word("well‑known"), Some("well-known".into()));
    assert_eq!(canonical_word("CAFÉ"), Some("café".into()));
}

#[test]
fn alphanumeric_and_symbol_terms_are_not_dictionary_gated() {
    assert_eq!(
        keys("Zyxwovium R2D2 3D GPT-4 B2B C++ c++ C# F# 42"),
        ["zyxwovium", "r2d2", "3d", "gpt-4", "b2b", "c++", "c#", "f#"]
    );
    assert_eq!(keys("C+++ C++Builder F## next"), ["next"]);
}

#[test]
fn long_synthetic_text_has_deterministic_bounded_output() {
    let text = "Alpha don't العربية Beta-2 https://example.com/path snake_case ".repeat(20_000);
    assert_eq!(keys(&text), ["alpha", "don't", "beta-2"]);
}
