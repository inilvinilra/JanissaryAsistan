use crate::models::*;
use anyhow::{Context, Result};
use regex::Regex;
use std::path::Path;
use unicode_normalization::UnicodeNormalization;

pub fn parse_file(path: &str) -> Result<Document> {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // heading_source keeps original formatting (e.g. Markdown "#" marks) for heading
    // detection; raw_text is the cleaned version used for word/keyword analysis.
    let (raw_text, heading_source) = match ext.as_str() {
        "pdf" => {
            let text = extract_pdf(path)?;
            (text.clone(), text)
        }
        "txt" => {
            let text = extract_txt(path)?;
            (text.clone(), text)
        }
        "md" | "markdown" => extract_markdown(path)?,
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported file type: {}. Supported: pdf, txt, md",
                ext
            ))
        }
    };

    let file_type = match ext.as_str() {
        "pdf" => FileType::Pdf,
        "txt" => FileType::Txt,
        _ => FileType::Markdown,
    };

    analyze_text(path, file_type, raw_text, &heading_source)
}

fn extract_pdf(path: &str) -> Result<String> {
    let bytes = std::fs::read(path).context("Could not read PDF file")?;
    pdf_extract::extract_text_from_mem(&bytes).context("Could not extract PDF text")
}

fn extract_txt(path: &str) -> Result<String> {
    std::fs::read_to_string(path).context("Could not read TXT file")
}

// Returns (plain_text, original_markdown). The original is kept separately because
// converting to HTML and stripping tags also strips "#" marks, which heading
// detection relies on.
fn extract_markdown(path: &str) -> Result<(String, String)> {
    let content = std::fs::read_to_string(path).context("Could not read Markdown file")?;

    let parser = pulldown_cmark::Parser::new(&content);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);

    let tag_re = Regex::new(r"<[^>]+>")?;
    let plain = tag_re.replace_all(&html_output, " ").to_string();

    Ok((plain, content))
}

fn analyze_text(
    filename: &str,
    file_type: FileType,
    raw_text: String,
    heading_source: &str,
) -> Result<Document> {
    let normalized = raw_text.nfc().collect::<String>();
    let heading_source_normalized = heading_source.nfc().collect::<String>();
    let lower = normalized.to_lowercase();

    let word_count = normalized.split_whitespace().count();
    let headings = extract_headings(&heading_source_normalized);
    let keywords = extract_keywords(&normalized);
    let references = extract_references(&normalized);
    let language = detect_language(&lower);
    let sections = extract_sections(&heading_source_normalized);

    let has_abstract = lower.contains("özet") || lower.contains("abstract");
    let has_conclusion =
        lower.contains("sonuç") || lower.contains("conclusion") || lower.contains("result");
    let has_methodology =
        lower.contains("yöntem") || lower.contains("method") || lower.contains("methodology");
    let has_references = !references.is_empty()
        || lower.contains("kaynakça")
        || lower.contains("references")
        || lower.contains("bibliography");

    Ok(Document {
        filename: Path::new(filename)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(filename)
            .to_string(),
        file_type,
        raw_text: normalized,
        word_count,
        headings,
        keywords,
        references,
        has_references,
        has_abstract,
        has_conclusion,
        has_methodology,
        language,
        sections,
    })
}

fn extract_headings(text: &str) -> Vec<String> {
    let mut headings = Vec::new();

    let md_re = Regex::new(r"^#{1,6}\s+(.+)$").unwrap();
    let num_re = Regex::new(r"^\d+(\.\d+)*\s+[A-ZÇĞIİÖŞÜ][^.!?]{3,50}$").unwrap();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(cap) = md_re.captures(trimmed) {
            headings.push(cap[1].trim().to_string());
        } else if num_re.is_match(trimmed) {
            headings.push(trimmed.to_string());
        }
    }

    headings.dedup();
    headings
}

// Simple TF-IDF-like keyword extraction. The stop-word/indicator lists below are
// Turkish and English language data (not code naming) — the parser is expected to
// handle documents in either language.
pub fn extract_keywords(text: &str) -> Vec<String> {
    let stop_words: std::collections::HashSet<&str> = [
        "ve", "veya", "ile", "için", "bu", "bir", "de", "da", "den", "dan", "en", "çok", "daha",
        "olan", "olarak", "ise", "ancak", "fakat", "the", "a", "an", "and", "or", "in", "on",
        "at", "to", "for", "of", "is", "are", "was", "were", "be", "been", "have", "has", "had",
        "that", "this", "these", "those", "with", "from", "by", "as",
    ]
    .iter()
    .copied()
    .collect();

    let word_re = Regex::new(r"\b[a-zA-ZçğıiöşüÇĞİÖŞÜ]{4,}\b").unwrap();
    let mut freq: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for word in word_re.find_iter(&text.to_lowercase()) {
        let w = word.as_str().to_string();
        if !stop_words.contains(w.as_str()) {
            *freq.entry(w).or_insert(0) += 1;
        }
    }

    let mut sorted: Vec<(String, usize)> = freq.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.into_iter().take(30).map(|(w, _)| w).collect()
}

fn extract_references(text: &str) -> Vec<String> {
    let mut refs = Vec::new();

    let doi_re = Regex::new(r"10\.\d{4,}/[^\s]+").unwrap();
    let url_re = Regex::new(r"https?://[^\s,;)]+").unwrap();
    let bracket_re = Regex::new(r"\[\d+\]").unwrap();

    for doi in doi_re.find_iter(text) {
        refs.push(format!("DOI: {}", doi.as_str()));
    }
    for url in url_re.find_iter(text) {
        refs.push(url.as_str().to_string());
    }
    for br in bracket_re.find_iter(text) {
        refs.push(br.as_str().to_string());
    }

    refs.dedup();
    refs.into_iter().take(50).collect()
}

// Whole-word matching, not substring — "proje" must not match inside "project".
fn detect_language(text: &str) -> Language {
    let words: std::collections::HashSet<&str> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();

    let turkish_indicators = ["için", "olan", "veya", "ancak", "çünkü", "çalışma", "proje"];
    let english_indicators = [
        "however",
        "therefore",
        "furthermore",
        "research",
        "study",
        "analysis",
    ];

    let tr_count = turkish_indicators.iter().filter(|w| words.contains(*w)).count();
    let en_count = english_indicators.iter().filter(|w| words.contains(*w)).count();

    if tr_count > en_count {
        Language::Turkish
    } else if en_count > tr_count {
        Language::English
    } else {
        Language::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_english_without_false_positive_on_project() {
        let text = "this project is for analysis, however the study needs more research";
        assert!(matches!(detect_language(text), Language::English));
    }

    #[test]
    fn detects_turkish() {
        let text = "bu proje için çalışma yapıldı ancak veya olan durumlar önemli";
        assert!(matches!(detect_language(text), Language::Turkish));
    }
}

fn extract_sections(text: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let heading_re = Regex::new(r"(?m)^(#{1,3}\s+.+|^\d+\.\s+[A-ZÇĞIİÖŞÜ].+)$").unwrap();

    let positions: Vec<_> = heading_re.find_iter(text).collect();

    for (i, m) in positions.iter().enumerate() {
        let title = m.as_str().trim().to_string();
        let start = m.end();
        let end = if i + 1 < positions.len() {
            positions[i + 1].start()
        } else {
            text.len()
        };

        let content = text[start..end].trim().to_string();
        let word_count = content.split_whitespace().count();

        if !content.is_empty() {
            sections.push(Section {
                title,
                content,
                word_count,
            });
        }
    }

    sections
}
