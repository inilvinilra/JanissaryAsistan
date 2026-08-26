use super::*;
use crate::models::{FileType, KpiTemplate, Language};

/// Convenience wrapper for the tests: production compares prepared token sets
/// so each report is tokenised once across a whole competition.
fn analyze_project_similarity(left: &Document, right: &Document) -> ProjectSimilarityResult {
    compare_similarity_tokens(&similarity_tokens(left), &similarity_tokens(right))
}

fn document(text: &str, keywords: &[&str]) -> Document {
    Document {
        filename: "submission.pdf".into(),
        file_type: FileType::Pdf,
        raw_text: text.into(),
        word_count: text.split_whitespace().count(),
        headings: Vec::new(),
        keywords: keywords.iter().map(|keyword| (*keyword).into()).collect(),
        references: Vec::new(),
        has_references: false,
        has_abstract: false,
        has_conclusion: false,
        has_methodology: false,
        language: Language::english(),
        sections: Vec::new(),
    }
}

fn category(name: &str, kpi: &str) -> CategoryTemplate {
    CategoryTemplate {
        category: name.into(),
        kpis: vec![KpiTemplate {
            name: kpi.into(),
            weight: 100.0,
            description: kpi.into(),
        }],
    }
}

#[test]
fn category_fit_recommends_the_best_matching_category() {
    let categories = vec![
        category("robotics", "autonomous robot navigation"),
        category("health", "clinical diagnostic patient safety"),
    ];
    let result = analyze_category_fit(
        &document(
            "An autonomous robot performs navigation with a vision sensor.",
            &["robot"],
        ),
        "health",
        &categories,
    )
    .expect("categories are available");

    assert_eq!(result.recommended_category, "robotics");
    assert!(result.recommended_category_score > result.current_category_score);
    assert!(result.requires_review);
}

#[test]
fn category_fit_keeps_current_category_when_it_has_the_best_evidence() {
    let categories = vec![
        category("robotics", "autonomous robot navigation"),
        category("health", "clinical diagnostic patient safety"),
    ];
    let result = analyze_category_fit(
        &document(
            "Clinical diagnostic software improves patient safety.",
            &["clinical"],
        ),
        "health",
        &categories,
    )
    .expect("categories are available");

    assert_eq!(result.recommended_category, "health");
    assert!(!result.requires_review);
}

#[test]
fn project_similarity_is_symmetric_and_explainable() {
    let left = document(
        "Autonomous robot navigation uses a vision sensor.",
        &["robotics"],
    );
    let right = document(
        "A robot uses vision navigation for autonomous movement.",
        &["robotics"],
    );

    let forward = analyze_project_similarity(&left, &right);
    let reverse = analyze_project_similarity(&right, &left);

    assert_eq!(forward, reverse);
    assert!(forward.similarity > 0.2);
    assert!(forward.matched_terms.contains(&"robot".to_string()));
}

#[test]
fn unrelated_projects_do_not_share_a_high_similarity_score() {
    let left = document("Autonomous robot navigation uses a vision sensor.", &[]);
    let right = document("Water quality is measured in agricultural irrigation.", &[]);

    assert!(analyze_project_similarity(&left, &right).similarity < 0.1);
}

fn turkish_document(text: &str) -> Document {
    Document {
        filename: "rapor.md".into(),
        file_type: FileType::Markdown,
        raw_text: text.into(),
        word_count: text.split_whitespace().count(),
        headings: Vec::new(),
        keywords: Vec::new(),
        references: Vec::new(),
        has_references: false,
        has_abstract: false,
        has_conclusion: false,
        has_methodology: false,
        language: Language::turkish(),
        sections: Vec::new(),
    }
}

fn template_for(category: &str) -> CategoryTemplate {
    CategoryTemplate {
        category: category.into(),
        kpis: vec![KpiTemplate {
            name: "Innovation".into(),
            weight: 100.0,
            description: "Novelty of the approach compared to existing solutions".into(),
        }],
    }
}

fn all_templates() -> Vec<CategoryTemplate> {
    [
        "ai",
        "cybersecurity",
        "software",
        "sustainability",
        "health-tech",
        "robotics",
    ]
    .iter()
    .map(|category| template_for(category))
    .collect()
}

/// Before the bilingual vocabulary every Turkish report scored 0% against its
/// own category and the recommendation was the same for all of them.
#[test]
fn a_turkish_cybersecurity_report_is_matched_to_cybersecurity() {
    let document = turkish_document(
        "Bu proje kurumsal ağlarda fidye yazılımı saldırılarını erken tespit eden bir          güvenlik motoru geliştirmektedir. Şifreleme davranışı izlenmekte, şüpheli süreç          karantinaya alınmaktadır. Zafiyet taraması ve kimlik doğrulama da desteklenir.",
    );
    let result = analyze_category_fit(&document, "cybersecurity", &all_templates()).unwrap();
    assert_eq!(result.recommended_category, "cybersecurity");
    assert!(
        result.current_category_score > 0.0,
        "kendi kategorisi sıfır puan aldı"
    );
    assert!(
        !result.requires_review,
        "doğru kategori incelemeye düşmemeli"
    );
}

#[test]
fn a_turkish_irrigation_report_is_not_matched_to_cybersecurity() {
    let document = turkish_document(
        "Tarımsal sulama sistemlerinde su tüketimini azaltmak için toprak nem sensörleri          kullanılmaktadır. Sulama takvimi iklim verisiyle güncellenmekte, enerji güneş          panelinden sağlanmaktadır. Sürdürülebilir su kullanımı hedeflenmektedir.",
    );
    let result = analyze_category_fit(&document, "sustainability", &all_templates()).unwrap();
    assert_eq!(result.recommended_category, "sustainability");
    assert!(result.current_category_score > 0.0);
}

/// A report filed under the wrong category must be flagged for the jury.
#[test]
fn a_misfiled_report_is_flagged_for_review() {
    let document = turkish_document(
        "Bu proje kurumsal ağlarda fidye yazılımı saldırılarını erken tespit eden bir          güvenlik motoru geliştirmektedir. Şifreleme davranışı izlenmekte, şüpheli süreç          karantinaya alınmaktadır. Zafiyet taraması ve kimlik doğrulama da desteklenir.",
    );
    let result = analyze_category_fit(&document, "health-tech", &all_templates()).unwrap();
    assert_eq!(result.recommended_category, "cybersecurity");
    assert!(
        result.requires_review,
        "yanlış kategoriye yüklenen rapor işaretlenmeli"
    );
}

/// Turkish suffixes must not hide a keyword: "sulama" has to match "sulamada".
#[test]
fn inflected_turkish_forms_still_match_their_keyword() {
    let inflected = turkish_document(
        "Sulamada kullanılan sensörler güneşten beslenmekte, iklimlendirme verileriyle          sürdürülebilirlik hedefine katkı sağlamaktadır. Karbon salımı azaltılmaktadır.",
    );
    let result = analyze_category_fit(&inflected, "sustainability", &all_templates()).unwrap();
    assert_eq!(result.recommended_category, "sustainability");
}

/// A short report copied verbatim into a much longer document, padded with
/// unrelated filler, must still be caught — the containment coefficient stays
/// high even though the padding drags Jaccard down.
#[test]
fn a_verbatim_copy_padded_with_filler_is_still_flagged() {
    let short = document(
        "Bu proje tarımsal sulama sistemlerinde su tüketimini azaltmak amacıyla toprak nem \
         sensörleri ve hava durumu verilerini birleştiren bir karar destek sistemi \
         geliştirmektedir. Sistem sulama zamanlamasını optimize ederek su kullanımını \
         düşürmektedir.",
        &[],
    );
    let filler = " Bu bölümde ayrıca ekip üyelerinin geçmiş deneyimleri, kullanılan yazılım \
         araçları, proje takvimi, bütçe planlaması, risk yönetimi süreçleri, paydaş iletişimi, \
         kalite güvence adımları, test senaryoları, dağıtım stratejisi ve gelecek geliştirme \
         planları detaylı biçimde ele alınmaktadır. Ekip haftalık toplantılarla ilerlemeyi takip \
         etmiş, her sprint sonunda değerlendirme yapmıştır. Dokümantasyon süreci boyunca \
         güncellenmiş, paydaşlarla düzenli geri bildirim alışverişinde bulunulmuştur."
        .repeat(3);
    let padded = document(&format!("{}{filler}", short.raw_text), &[]);

    let result = analyze_project_similarity(&short, &padded);
    assert!(
        result.similarity >= 0.45,
        "içerme ölçütü şişirilmiş kopyayı yakalamalı, çıkan: {:.3}",
        result.similarity
    );
}

/// Turkish suffixes must not hide a near-duplicate: the same sentence reworded
/// with different case endings should still register as highly similar.
#[test]
fn inflected_turkish_rewording_is_still_recognised_as_similar() {
    let original = document(
        "Sulama sistemleri toprak nem sensörlerini kullanarak su tüketimini azaltır ve \
         çiftçilere sulama zamanlamasında yardımcı olur.",
        &[],
    );
    let reworded = document(
        "Sulamada toprak nem sensörlerinin kullanılmasıyla su tüketimi azalmakta, \
         çiftçiler sulamayı zamanlamada yardım almaktadır.",
        &[],
    );

    let result = analyze_project_similarity(&original, &reworded);
    assert!(
        result.similarity > 0.4,
        "çekim ekleri örtüşmeyi gizlememeli, çıkan: {:.3}",
        result.similarity
    );
}

/// Two unrelated same-category reports share generic connector words ("için",
/// "the", "ve") that must not, by themselves, register as similarity.
#[test]
fn shared_stopwords_between_unrelated_reports_do_not_inflate_similarity() {
    let left = document(
        "Bu proje için geliştirilen sistem, kullanıcıların ve yöneticilerin ihtiyaçlarını \
         karşılamak amacıyla tasarlanmıştır ve bu sayede verimliliği artırmaktadır.",
        &[],
    );
    let right = document(
        "Bu çalışma için hazırlanan model, öğrencilerin ve öğretmenlerin taleplerini \
         gözetmek üzere kurgulanmıştır ve bu doğrultuda başarıyı yükseltmektedir.",
        &[],
    );

    let result = analyze_project_similarity(&left, &right);
    assert_eq!(
        result.similarity, 0.0,
        "iki metnin ortak hiçbir kelimesi olmamalı, çıkan: {:?}",
        result.matched_terms
    );
}

/// Shared report-template headings ("Özet", "Sonuç") must not count as
/// similarity — every compliant submission has them regardless of subject.
#[test]
fn shared_template_section_titles_do_not_count_as_similarity() {
    let left = document(
        "# Özet\nBu bölümde havacılık motorlarının yakıt verimliliği incelenmektedir.\n\
         # Sonuç\nMotor tasarımı önerilen değişikliklerle geliştirilmiştir.",
        &[],
    );
    let right = document(
        "# Özet\nBu bölümde deniz canlılarının göç davranışları incelenmektedir.\n\
         # Sonuç\nGöç güzergahları uydu verileriyle haritalanmıştır.",
        &[],
    );

    let result = analyze_project_similarity(&left, &right);
    assert!(
        !result
            .matched_terms
            .iter()
            .any(|term| term == "ozet" || term == "sonuc"),
        "başlık kelimeleri eşleşen terimlerde görünmemeli: {:?}",
        result.matched_terms
    );
}
