use crate::models::Language;

pub(crate) const TURKISH_STOPWORDS: [&str; 34] = [
    "ve",
    "ile",
    "için",
    "olan",
    "bu",
    "bir",
    "olarak",
    "daha",
    "veya",
    "ancak",
    "çünkü",
    "gibi",
    "kadar",
    "sonra",
    "üzere",
    "tarafından",
    "ayrıca",
    "ise",
    "her",
    "çok",
    "göre",
    "arasında",
    "ilgili",
    "hem",
    "bunun",
    "olduğu",
    "yapılan",
    "elde",
    "edilen",
    "bölümde",
    "yöntem",
    "çalışma",
    "proje",
    "sistem",
];

pub(crate) const ENGLISH_STOPWORDS: [&str; 30] = [
    "the", "and", "of", "to", "in", "for", "with", "that", "this", "are", "is", "was", "were",
    "on", "as", "by", "from", "which", "has", "have", "been", "not", "but", "their", "its", "can",
    "will", "between", "during", "our",
];

const MIN_LETTERS: usize = 24;

fn turkish_letter_signal(lowered: &str) -> f64 {
    let letters = lowered
        .chars()
        .filter(|character| character.is_alphabetic())
        .count();
    if letters == 0 {
        return 0.0;
    }
    let distinctive = lowered
        .chars()
        .filter(|character| matches!(character, 'ğ' | 'ı' | 'ş'))
        .count() as f64;
    let shared = lowered
        .chars()
        .filter(|character| matches!(character, 'ç' | 'ö' | 'ü'))
        .count() as f64;
    (distinctive * 3.0 + shared * 0.3) / letters as f64
}

fn stopword_ratio(words: &[&str], list: &[&str]) -> f64 {
    if words.is_empty() {
        return 0.0;
    }
    words.iter().filter(|word| list.contains(word)).count() as f64 / words.len() as f64
}

fn builtin_verdict(text: &str) -> Option<Language> {
    match whatlang::detect(text) {
        Some(info) if info.is_reliable() => Some(Language::new(info.lang().eng_name())),
        _ => None,
    }
}

pub fn detect(text: &str) -> Language {
    let lowered = text.to_lowercase();
    if lowered
        .chars()
        .filter(|character| character.is_alphabetic())
        .count()
        < MIN_LETTERS
    {
        return Language::unknown();
    }
    let words: Vec<&str> = lowered
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();

    let azerbaijani_marker = lowered.contains('ə');
    let turkish_score =
        turkish_letter_signal(&lowered) * 4.0 + stopword_ratio(&words, &TURKISH_STOPWORDS);
    if turkish_score >= 0.08 && !azerbaijani_marker {
        return Language::turkish();
    }

    if let Some(verdict) = builtin_verdict(text) {
        return verdict;
    }
    if turkish_score >= 0.03 {
        return Language::turkish();
    }
    if stopword_ratio(&words, &ENGLISH_STOPWORDS) >= 0.12 {
        return Language::english();
    }
    Language::unknown()
}

pub fn supported_names() -> Vec<String> {
    let mut names = whatlang::Lang::all()
        .iter()
        .map(|language| language.eng_name().to_string())
        .collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();
    names
}

#[cfg(test)]
#[path = "language_tests.rs"]
mod tests;
