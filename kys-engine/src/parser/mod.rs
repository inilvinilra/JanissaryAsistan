// JanissaryAsistan - Dosya Ayrıştırma Motoru
// PDF, TXT, Markdown dosyalarını okur ve yapılandırılmış Document döndürür

use crate::models::*;
use anyhow::{Context, Result};
use regex::Regex;
use std::path::Path;
use tracing::info;
use unicode_normalization::UnicodeNormalization;

/// Dosya uzantısına göre doğru parser'ı çağırır
pub fn parse_file(path: &str) -> Result<Document> {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    info!("Dosya tipi tespit edildi: {}", ext);

    let raw_text = match ext.as_str() {
        "pdf" => extract_pdf(path)?,
        "txt" => extract_txt(path)?,
        "md" | "markdown" => extract_markdown(path)?,
        _ => {
            return Err(anyhow::anyhow!(
                "Desteklenmeyen dosya tipi: {}. Desteklenenler: pdf, txt, md",
                ext
            ))
        }
    };

    let file_type = match ext.as_str() {
        "pdf" => FileType::Pdf,
        "txt" => FileType::Txt,
        _ => FileType::Markdown,
    };

    analyze_text(path, file_type, raw_text)
}

/// PDF'den metin çıkarır
fn extract_pdf(path: &str) -> Result<String> {
    let bytes = std::fs::read(path).context("PDF dosyası okunamadı")?;
    let text = pdf_extract::extract_text_from_mem(&bytes)
        .context("PDF metni çıkarılamadı")?;
    Ok(text)
}

/// TXT dosyasını okur
fn extract_txt(path: &str) -> Result<String> {
    std::fs::read_to_string(path).context("TXT dosyası okunamadı")
}

/// Markdown'dan düz metin çıkarır
fn extract_markdown(path: &str) -> Result<String> {
    let content = std::fs::read_to_string(path).context("Markdown dosyası okunamadı")?;
    
    // Markdown'ı HTML'e çevir, sonra HTML taglerini temizle
    let parser = pulldown_cmark::Parser::new(&content);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);

    // HTML taglerini kaldır, düz metin bırak
    let tag_re = Regex::new(r"<[^>]+>")?;
    let plain = tag_re.replace_all(&html_output, " ");
    
    Ok(plain.to_string())
}

/// Ham metni analiz eder ve Document oluşturur
fn analyze_text(filename: &str, file_type: FileType, raw_text: String) -> Result<Document> {
    let normalized = raw_text.nfc().collect::<String>();
    let lower = normalized.to_lowercase();

    let word_count = normalized.split_whitespace().count();
    let headings = extract_headings(&normalized);
    let keywords = extract_keywords(&normalized);
    let references = extract_references(&normalized);
    let language = detect_language(&lower);
    let sections = extract_sections(&normalized);

    // Kritik bölümlerin varlığını kontrol et
    let has_abstract = lower.contains("özet") || lower.contains("abstract");
    let has_conclusion = lower.contains("sonuç") || lower.contains("conclusion") || lower.contains("result");
    let has_methodology = lower.contains("yöntem") || lower.contains("method") || lower.contains("methodology");
    let has_references = !references.is_empty() 
        || lower.contains("kaynakça") 
        || lower.contains("references")
        || lower.contains("bibliography");

    let author = extract_author(&normalized);

    Ok(Document {
        filename: Path::new(filename).file_name().unwrap_or_default().to_string_lossy().to_string(),
        file_type: file_type,
        raw_text: normalized,
        word_count,
        headings,
        keywords,
        references,
        has_references,
        has_bibliography: false,
        reference_count: 0,
        classified_category: None,
        has_abstract,
        has_conclusion,
        has_methodology,
        language,
        sections,
        author,
    })
}

/// PDF kapak/ilk sayfasından yazar veya takım kaptanını çıkarır
fn extract_author(text: &str) -> Option<String> {
    let first_page = text.chars().take(2000).collect::<String>();
    
    // Daha esnek Regex: iki nokta şart değil, yeni satır \n kabul ediyor, 1-4 kelimeli isimler
    let re = regex::Regex::new(r"(?i)(?:takım kaptanı|hazırlayanlar|hazırlayan|yazar|danışman|öğrenci|takım üyeleri|proje sahibi|proje yürütücüsü)(?:[:\s\r\n]+)([A-ZÇĞIİÖŞÜ][a-zçğıiöşü]+(?:[ \t\r\n]+[A-ZÇĞIİÖŞÜ][a-zçğıiöşü]+){1,4})").unwrap();
    
    if let Some(caps) = re.captures(&first_page) {
        let raw_name = caps[1].trim().to_string();
        // İsimdeki alt satır ve fazla boşlukları temizle
        let clean_name = regex::Regex::new(r"\s+").unwrap().replace_all(&raw_name, " ").to_string();
        return Some(clean_name);
    }
    
    None
}

/// Başlıkları tespit eder (büyük harfli satırlar, markdown #, numaralı bölümler)
fn extract_headings(text: &str) -> Vec<String> {
    let mut headings = Vec::new();

    // Markdown başlıkları: # Başlık, ## Alt Başlık
    let md_re = Regex::new(r"^#{1,6}\s+(.+)$").unwrap();
    // Numaralı bölümler: "1. Giriş", "2.1 Yöntem"
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

/// Anahtar kelimeleri çıkarır (TF-IDF benzeri basit yaklaşım)
pub fn extract_keywords(text: &str) -> Vec<String> {
    // Türkçe ve İngilizce stop words
    let stop_words: std::collections::HashSet<&str> = [
        "ve", "veya", "ile", "için", "bu", "bir", "de", "da", "den", "dan",
        "en", "çok", "daha", "olan", "olan", "olarak", "ise", "ancak", "fakat",
        "the", "a", "an", "and", "or", "in", "on", "at", "to", "for", "of",
        "is", "are", "was", "were", "be", "been", "have", "has", "had",
        "that", "this", "these", "those", "with", "from", "by", "as",
    ].iter().copied().collect();

    let word_re = Regex::new(r"\b[a-zA-ZçğıiöşüÇĞİÖŞÜ]{4,}\b").unwrap();
    let mut freq: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for word in word_re.find_iter(&text.to_lowercase()) {
        let w = word.as_str().to_string();
        if !stop_words.contains(w.as_str()) {
            *freq.entry(w).or_insert(0) += 1;
        }
    }

    // En sık geçen 30 kelimeyi döndür
    let mut sorted: Vec<(String, usize)> = freq.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.into_iter().take(30).map(|(w, _)| w).collect()
}

/// Referansları/kaynakçayı çıkarır
fn extract_references(text: &str) -> Vec<String> {
    let mut refs = Vec::new();
    
    // DOI pattern
    let doi_re = Regex::new(r"10\.\d{4,}/[^\s]+").unwrap();
    // URL pattern
    let url_re = Regex::new(r"https?://[^\s,;)]+").unwrap();
    // Klasik akademik referans: [1], (Smith, 2020), vb.
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

/// Dil tespiti (basit keyword bazlı)
fn detect_language(text: &str) -> Language {
    let turkish_indicators = ["için", "olan", "veya", "ancak", "çünkü", "çalışma", "proje"];
    let english_indicators = ["however", "therefore", "furthermore", "research", "study", "analysis"];

    let tr_count: usize = turkish_indicators.iter().filter(|w| text.contains(*w)).count();
    let en_count: usize = english_indicators.iter().filter(|w| text.contains(*w)).count();

    if tr_count > en_count {
        Language::Turkish
    } else if en_count > tr_count {
        Language::English
    } else {
        Language::Unknown
    }
}

/// Metni bölümlere ayırır
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
            sections.push(Section { title, content, word_count });
        }
    }

    sections
}
