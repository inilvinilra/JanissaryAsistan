// KYS - Analiz ve Puanlama Motoru
// Benzerlik hesaplama + kural tabanlı değerlendirme

use crate::models::*;
use crate::parser::extract_keywords;
use std::collections::HashSet;

pub mod taxonomy;

/// Proje belgesi ile internet kaynakları arasındaki benzerliği hesaplar
pub fn compute_similarity(
    document: &Document,
    search_results: &[SearchResult],
) -> SimilarityReport {
    let doc_keywords: HashSet<String> = document.keywords.iter().cloned().collect();
    let mut matches = Vec::new();

    for result in search_results {
        if let Some(content) = &result.fetched_content {
            let source_keywords: HashSet<String> = extract_keywords(content).into_iter().collect();

            // Jaccard benzerliği: kesişim / birleşim
            let intersection: HashSet<&String> = doc_keywords.intersection(&source_keywords).collect();
            let union: HashSet<&String> = doc_keywords.union(&source_keywords).collect();

            let similarity_score = if union.is_empty() {
                0.0
            } else {
                intersection.len() as f64 / union.len() as f64
            };

            let matched_keywords: Vec<String> = intersection
                .into_iter()
                .take(5)
                .cloned()
                .collect();

            let explanation = generate_similarity_explanation(similarity_score, &matched_keywords);

            matches.push(SimilarityMatch {
                title: result.title.clone(),
                url: result.url.clone(),
                source_type: result.source_type.clone(),
                similarity_score,
                matched_keywords,
                explanation,
            });
        }
    }

    // En yüksek benzerlikten düşüğe sırala
    matches.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());

    let overall_score = if matches.is_empty() {
        0.0
    } else {
        let top_scores: Vec<f64> = matches.iter().take(3).map(|m| m.similarity_score).collect();
        top_scores.iter().sum::<f64>() / top_scores.len() as f64
    };

    SimilarityReport {
        overall_score,
        matches,
    }
}

/// Benzerlik açıklaması üretir (template tabanlı)
fn generate_similarity_explanation(score: f64, matched_keywords: &[String]) -> String {
    let kw_str = matched_keywords.join(", ");

    match score {
        s if s < 0.10 => format!(
            "Düşük teknik örtüşme. Ortak terimler: {}",
            if kw_str.is_empty() { "bulunamadı".to_string() } else { kw_str }
        ),
        s if s < 0.25 => format!(
            "Benzer teknik yaklaşımlar bulundu. Örtüşen kavramlar: {}",
            kw_str
        ),
        s if s < 0.45 => format!(
            "Benzer yöntem kullanan çalışma tespit edildi. Ortak alanlar: {}",
            kw_str
        ),
        s if s < 0.65 => format!(
            "Bu problem daha önce benzer şekilde ele alınmış. İlgili kavramlar: {}",
            kw_str
        ),
        _ => format!(
            "Yüksek teknik benzerlik. Ortak terimler: {}. Jüri incelemesi önerilir.",
            kw_str
        ),
    }
}

/// Kural tabanlı belge puanlaması (AI olmadan)
pub fn score_document(document: &Document, category_fit: f64, classified_category: Option<String>) -> ScoreCard {
    ScoreCard {
        category_fit,
        classified_category,
        completeness: score_completeness(document),
        reference_quality: score_references(document),
        technical_depth: score_technical_depth(document),
        originality: 75.0, // Varsayılan - benzerlik analizi sonrası güncellenir
    }
}



/// Bölüm tamlığı: özet, giriş, sonuç, yöntem var mı?
fn score_completeness(doc: &Document) -> f64 {
    let mut score: f64 = 0.0;
    let max: f64 = 100.0;

    // Her kritik bölüm +25 puan
    if doc.has_abstract {
        score += 25.0;
    }
    if doc.has_conclusion {
        score += 25.0;
    }
    if doc.has_methodology {
        score += 25.0;
    }
    if doc.has_references {
        score += 25.0;
    }

    // Kelime sayısı bonusu
    let word_bonus: f64 = match doc.word_count {
        0..=500 => -10.0,
        501..=1000 => 0.0,
        1001..=3000 => 5.0,
        _ => 10.0,
    };

    (score + word_bonus).max(0.0_f64).min(max)
}

/// Kaynak kalitesi: kaynakça dolu mu?
fn score_references(doc: &Document) -> f64 {
    if !doc.has_references {
        return 20.0; // Kaynakça yok = çok düşük puan
    }

    let ref_count = doc.references.len();
    
    // DOI içeren referanslar daha değerli
    let doi_count = doc.references.iter()
        .filter(|r| r.starts_with("DOI:"))
        .count();

    let base = match ref_count {
        0 => 20.0,
        1..=3 => 45.0,
        4..=7 => 65.0,
        8..=15 => 80.0,
        _ => 90.0,
    };

    let doi_bonus = (doi_count as f64 * 5.0).min(10.0);
    (base + doi_bonus).min(100.0)
}

/// Teknik derinlik: kelime sayısı + bölüm sayısı + referans yoğunluğu
fn score_technical_depth(doc: &Document) -> f64 {
    let word_score: f64 = match doc.word_count {
        0..=300 => 20.0,
        301..=800 => 40.0,
        801..=2000 => 60.0,
        2001..=5000 => 75.0,
        _ => 85.0,
    };

    let section_score: f64 = match doc.sections.len() {
        0..=1 => 0.0,
        2..=3 => 10.0,
        4..=6 => 15.0,
        _ => 20.0,
    };

    // Ortalama bölüm uzunluğu bonusu
    let avg_section_words = if doc.sections.is_empty() {
        0
    } else {
        doc.sections.iter().map(|s| s.word_count).sum::<usize>() / doc.sections.len()
    };

    let depth_bonus: f64 = match avg_section_words {
        0..=50 => -5.0,
        51..=150 => 0.0,
        151..=400 => 5.0,
        _ => 10.0,
    };

    (word_score + section_score + depth_bonus).max(0.0_f64).min(100.0_f64)
}

/// Benzerlik raporundan özgünlük puanı üretir (0-100, yüksek = özgün)
pub fn originality_from_similarity(similarity: &SimilarityReport) -> f64 {
    // Benzerlik yüksekse özgünlük düşük
    let base = (1.0 - similarity.overall_score) * 100.0;
    base.max(0.0).min(100.0)
}

/// Puanı günceller - benzerlik analizi tamamlandıktan sonra çağrılır
pub fn update_score_with_similarity(score: &mut ScoreCard, similarity: &SimilarityReport) {
    score.originality = originality_from_similarity(similarity);
}
