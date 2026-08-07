use std::fs;
use std::path::Path;

pub struct Taxonomy {
    pub categories: Vec<Category>,
}

pub struct Category {
    pub name: String,
    pub subcategories: Vec<SubCategory>,
}

pub struct SubCategory {
    pub name: String,
    pub keywords: Vec<String>,
}

impl Taxonomy {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Self {
        let content = fs::read_to_string(path).unwrap_or_default();
        Self::parse(&content)
    }

    fn parse(content: &str) -> Self {
        let mut categories = Vec::new();
        let mut current_category: Option<Category> = None;
        let mut current_subcategory: Option<SubCategory> = None;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("# ") {
                if let Some(mut cat) = current_category.take() {
                    if let Some(sub) = current_subcategory.take() {
                        cat.subcategories.push(sub);
                    }
                    categories.push(cat);
                }
                current_category = Some(Category {
                    name: line.trim_start_matches("# ").trim().to_string(),
                    subcategories: Vec::new(),
                });
            } else if line.starts_with("## ") {
                if let Some(mut sub) = current_subcategory.take() {
                    if let Some(cat) = current_category.as_mut() {
                        cat.subcategories.push(sub);
                    }
                }
                current_subcategory = Some(SubCategory {
                    name: line.trim_start_matches("## ").trim().to_string(),
                    keywords: Vec::new(),
                });
            } else if line.starts_with("- ") {
                let kw = line.trim_start_matches("- ").trim().to_lowercase();
                if let Some(sub) = current_subcategory.as_mut() {
                    sub.keywords.push(kw);
                } else if let Some(cat) = current_category.as_mut() {
                    // Fallback for keywords directly under H1
                    cat.subcategories.push(SubCategory {
                        name: "Genel".to_string(),
                        keywords: vec![kw],
                    });
                }
            }
        }
        
        if let Some(mut cat) = current_category.take() {
            if let Some(sub) = current_subcategory.take() {
                cat.subcategories.push(sub);
            }
            categories.push(cat);
        }

        Self { categories }
    }

    /// Sınıflandırma ve Alan Uyumu puanını hesaplar
    pub fn classify(&self, doc_keywords: &[String]) -> (Option<String>, f64) {
        if doc_keywords.is_empty() {
            return (None, 0.0);
        }

        let mut best_match: Option<String> = None;
        let mut highest_hits = 0;
        let mut total_matches = 0;

        let doc_keywords_lower: Vec<String> = doc_keywords.iter().map(|k| k.to_lowercase()).collect();

        for cat in &self.categories {
            for sub in &cat.subcategories {
                let mut hits = 0;
                for doc_kw in &doc_keywords_lower {
                    // Tam eşleşme veya parça eşleşmesi
                    if sub.keywords.iter().any(|k| k.contains(doc_kw) || doc_kw.contains(k)) {
                        hits += 1;
                        total_matches += 1;
                    } else if sub.name.to_lowercase().contains(doc_kw) || cat.name.to_lowercase().contains(doc_kw) {
                        hits += 1;
                        total_matches += 1;
                    }
                }

                if hits > highest_hits {
                    highest_hits = hits;
                    best_match = Some(format!("{} > {}", cat.name, sub.name));
                }
            }
        }

        if total_matches == 0 {
            return (None, 40.0); // Hiçbir kategoride bulunamadıysa uyum zayıftır.
        }

        // Odaklanma Oranı (Concentration): Kelimeler ne kadar tek bir alt kategoride toplanmış?
        // 1.0'a yaklaşması harika bir odak olduğunu gösterir.
        let concentration = highest_hits as f64 / total_matches as f64;
        
        // Taban puan 50. Odaklanmaya göre +50 puan eklenir.
        let score = 50.0 + (concentration * 50.0);

        (best_match, score)
    }
}
