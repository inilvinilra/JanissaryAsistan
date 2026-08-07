// KYS - Veri Modelleri
// Sistemin tüm katmanları bu yapıları kullanır

use serde::{Deserialize, Serialize};

/// Parse edilmiş belgeyi temsil eder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub filename: String,
    pub file_type: FileType,
    pub raw_text: String,
    pub word_count: usize,
    pub headings: Vec<String>,
    pub keywords: Vec<String>,
    pub references: Vec<String>,
    pub has_references: bool,
    pub has_abstract: bool,
    pub has_conclusion: bool,
    pub has_methodology: bool,
    pub language: Language,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    Pdf,
    Txt,
    Markdown,
    Docx,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Language {
    Turkish,
    English,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub title: String,
    pub content: String,
    pub word_count: usize,
}

/// İnternet araştırması sonucu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source_type: String, // "academic", "github", "documentation", "web"
    pub fetched_content: Option<String>,
    pub http_status: u16,
}

/// Benzerlik analizi sonucu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityMatch {
    pub title: String,
    pub url: String,
    pub source_type: String,
    pub similarity_score: f64, // 0.0 - 1.0
    pub matched_keywords: Vec<String>,
    pub explanation: String,
}

/// Tüm benzerlik analizi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityReport {
    pub overall_score: f64, // Ortalama benzerlik (yüksek = az özgün)
    pub matches: Vec<SimilarityMatch>,
}

impl SimilarityReport {
    pub fn originality_label(&self) -> &str {
        match self.overall_score {
            s if s < 0.20 => "✅ Özgün görünüyor - Literatürde doğrudan benzeri bulunamadı",
            s if s < 0.40 => "⚠️ Benzer teknik yaklaşımlar mevcut",
            s if s < 0.60 => "⚠️ Benzer yöntem kullanan çalışmalar bulundu",
            s if s < 0.80 => "🔴 Benzer problem daha önce çözülmüş",
            _ => "🔴 Literatürde yüksek benzerlik tespit edildi",
        }
    }
}

/// Değerlendirme puanları
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreCard {
    pub category_fit: f64,      // Kategori uyumu (0-100)
    pub completeness: f64,      // Bölüm tamlığı (0-100)
    pub reference_quality: f64, // Kaynak kalitesi (0-100)
    pub technical_depth: f64,   // Teknik derinlik (0-100)
    pub originality: f64,       // Özgünlük (0-100, similarity'den gelir)
}

impl ScoreCard {
    pub fn total(&self) -> f64 {
        self.category_fit * 0.20
            + self.completeness * 0.25
            + self.reference_quality * 0.20
            + self.technical_depth * 0.20
            + self.originality * 0.15
    }

    pub fn grade(&self) -> &str {
        match self.total() as u32 {
            95..=100 => "A+",
            90..=94 => "A",
            85..=89 => "B+",
            80..=84 => "B",
            75..=79 => "C+",
            70..=74 => "C",
            60..=69 => "D",
            _ => "F",
        }
    }

    pub fn reason(&self) -> String {
        let mut reasons = Vec::new();

        if self.completeness < 60.0 {
            reasons.push("Eksik bölümler tespit edildi".to_string());
        }
        if self.reference_quality < 50.0 {
            reasons.push("Kaynakça yetersiz veya eksik".to_string());
        }
        if self.technical_depth < 60.0 {
            reasons.push("Teknik derinlik yetersiz".to_string());
        }
        if self.originality < 50.0 {
            reasons.push("Benzer çalışmalar bulundu".to_string());
        }

        if reasons.is_empty() {
            "Proje genel kriterleri karşılıyor.".to_string()
        } else {
            reasons.join("; ")
        }
    }
}
