use super::*;
use crate::models::{FileType, Language, Section};

fn document(sections: Vec<(&str, usize)>, language: Language, words: usize) -> Document {
    Document {
        filename: "rapor.pdf".into(),
        file_type: FileType::Pdf,
        raw_text: String::new(),
        word_count: words,
        headings: sections
            .iter()
            .map(|(title, _)| title.to_string())
            .collect(),
        keywords: Vec::new(),
        references: Vec::new(),
        has_references: false,
        has_abstract: false,
        has_conclusion: false,
        has_methodology: false,
        language,
        sections: sections
            .into_iter()
            .map(|(title, word_count)| Section {
                title: title.into(),
                content: String::new(),
                word_count,
            })
            .collect(),
    }
}

fn template(sections: Vec<TemplateSection>) -> ReportTemplate {
    ReportTemplate {
        competition_id: 1,
        name: "TEKNOFEST Proje Raporu".into(),
        version: 3,
        expected_language: "Turkish".into(),
        min_words: 500,
        max_words: 10_000,
        sections,
        updated_at: String::new(),
        updated_by: String::new(),
    }
}

fn required(key: &str, title: &str, aliases: Vec<&str>, min_words: i64) -> TemplateSection {
    TemplateSection {
        key: key.into(),
        title: title.into(),
        aliases: aliases.into_iter().map(str::to_string).collect(),
        min_words,
        required: true,
    }
}

#[test]
fn normalization_folds_turkish_capitals_and_strips_numbering() {
    assert_eq!(normalize_heading("IÇINDEKILER"), "ıçındekıler");
    assert_eq!(normalize_heading("İÇİNDEKİLER"), "içindekiler");
    assert_eq!(normalize_heading("2. PROBLEM TANIMI"), "problem tanımı");
    assert_eq!(normalize_heading("1.1) Özgünlük"), "özgünlük");
    assert_eq!(normalize_heading("IV - Sonuç"), "sonuç");
}

#[test]
fn a_bare_ordinal_heading_is_not_stripped_into_nothing() {
    assert_eq!(normalize_heading("3."), "3");
}

#[test]
fn aliases_and_partial_titles_match_the_requirement() {
    let requirement = required("methodology", "Yöntem", vec!["Metodoloji", "Method"], 10);
    let candidates = vec![("2. METODOLOJİ VE YAKLAŞIM".to_string(), 400_i64)];
    let matched = best_match(&requirement, &candidates);
    assert!(matched.is_some());
    assert_eq!(matched.unwrap().0, "2. METODOLOJİ VE YAKLAŞIM");
}

#[test]
fn an_unrelated_heading_is_not_matched() {
    let requirement = required("methodology", "Yöntem", vec!["Metodoloji"], 10);
    let candidates = vec![("Bütçe ve Zaman Planı".to_string(), 400_i64)];
    assert!(best_match(&requirement, &candidates).is_none());
}

#[test]
fn a_fully_matching_report_is_compliant() {
    let sections = vec![
        required("abstract", "Özet", vec![], 10),
        required("conclusion", "Sonuç", vec![], 10),
    ];
    let doc = document(
        vec![("ÖZET", 120), ("4. SONUÇ", 90)],
        Language::turkish(),
        1200,
    );
    let result = evaluate(7, &template(sections), &doc);
    assert!(result.compliant, "{}", result.summary);
    assert_eq!(result.section_score, 100.0);
    assert!(result.language_matches);
}

#[test]
fn a_missing_required_section_fails_and_is_named() {
    let sections = vec![
        required("abstract", "Özet", vec![], 10),
        required("conclusion", "Sonuç", vec![], 10),
    ];
    let doc = document(vec![("ÖZET", 120)], Language::turkish(), 1200);
    let result = evaluate(7, &template(sections), &doc);
    assert!(!result.compliant);
    assert_eq!(result.section_score, 50.0);
    let conclusion = result
        .sections
        .iter()
        .find(|finding| finding.key == "conclusion")
        .unwrap();
    assert_eq!(conclusion.status, "missing");
    assert!(result.summary.contains("1 zorunlu başlık eksik"));
}

#[test]
fn a_present_but_short_section_is_reported_as_thin() {
    let sections = vec![required("abstract", "Özet", vec![], 100)];
    let doc = document(vec![("ÖZET", 12)], Language::turkish(), 1200);
    let result = evaluate(7, &template(sections), &doc);
    assert!(!result.compliant);
    assert_eq!(result.section_score, 50.0);
    assert_eq!(result.sections[0].status, "thin");
    assert!(result.summary.contains("beklenen içerikten kısa"));
}

#[test]
fn a_wrong_language_report_fails_even_with_every_section() {
    let sections = vec![required("abstract", "Özet", vec!["Abstract"], 10)];
    let doc = document(vec![("ABSTRACT", 300)], Language::english(), 1200);
    let result = evaluate(7, &template(sections), &doc);
    assert_eq!(result.section_score, 100.0);
    assert!(!result.compliant);
    assert!(!result.language_matches);
    assert!(result.summary.contains("rapor dili English"));
}

#[test]
fn word_count_limits_are_enforced_in_both_directions() {
    let sections = vec![required("abstract", "Özet", vec![], 10)];
    let short = document(vec![("ÖZET", 300)], Language::turkish(), 120);
    assert!(!evaluate(7, &template(sections.clone()), &short).word_count_within_range);
    let long = document(vec![("ÖZET", 300)], Language::turkish(), 40_000);
    assert!(!evaluate(7, &template(sections), &long).word_count_within_range);
}

#[test]
fn optional_sections_do_not_affect_the_score() {
    let sections = vec![
        required("abstract", "Özet", vec![], 10),
        TemplateSection {
            key: "references".into(),
            title: "Kaynakça".into(),
            aliases: vec![],
            min_words: 0,
            required: false,
        },
    ];
    let doc = document(vec![("ÖZET", 300)], Language::turkish(), 1200);
    let result = evaluate(7, &template(sections), &doc);
    assert_eq!(result.section_score, 100.0);
    assert!(result.compliant);
    assert_eq!(result.sections[1].status, "missing");
}

#[test]
fn markdown_markers_are_stripped_before_numbering() {
    assert_eq!(normalize_heading("# 1. ÖZET"), "özet");
    assert_eq!(normalize_heading("### IV - Sonuç"), "sonuç");
    assert_eq!(normalize_heading("## Kaynakça"), "kaynakça");
}

/// The parser reports section titles with their markdown prefix but lists
/// headings without it. The bare heading carries no word count and must not
/// win over the parsed section it duplicates.
#[test]
fn a_bare_heading_does_not_shadow_the_parsed_section_it_duplicates() {
    let sections = vec![required("abstract", "Özet", vec![], 80)];
    let mut doc = document(vec![("# 1. ÖZET", 168)], Language::turkish(), 1200);
    doc.headings = vec!["1. ÖZET".into()];
    let result = evaluate(7, &template(sections), &doc);
    assert_eq!(result.sections[0].status, "present");
    assert_eq!(result.sections[0].word_count, 168);
    assert!(result.compliant, "{}", result.summary);
}

#[test]
fn headings_without_parsed_sections_still_match_but_count_as_thin() {
    let sections = vec![required("abstract", "Özet", vec![], 50)];
    let mut doc = document(vec![], Language::turkish(), 1200);
    doc.headings = vec!["ÖZET".into()];
    let result = evaluate(7, &template(sections), &doc);
    assert_eq!(result.sections[0].status, "thin");
}

/// The readiness gate once tested `status != "passed"`, a value this module
/// never emits, so every required section counted as failing. The gate and this
/// module must agree on the vocabulary.
#[test]
fn only_present_sections_are_reported_as_satisfied() {
    let sections = vec![
        required("abstract", "Özet", vec![], 80),
        required("conclusion", "Sonuç", vec![], 80),
        required("method", "Yöntem", vec![], 80),
    ];
    let doc = document(vec![("ÖZET", 120), ("SONUÇ", 5)], Language::turkish(), 1200);
    let result = evaluate(7, &template(sections), &doc);

    let by_key = |key: &str| {
        result
            .sections
            .iter()
            .find(|finding| finding.key == key)
            .unwrap()
    };
    assert!(by_key("abstract").is_satisfied());
    assert!(
        !by_key("conclusion").is_satisfied(),
        "kısa bölüm sayılmamalı"
    );
    assert!(!by_key("method").is_satisfied(), "eksik bölüm sayılmamalı");

    let unsatisfied = result
        .sections
        .iter()
        .filter(|finding| finding.required && !finding.is_satisfied())
        .count();
    assert_eq!(unsatisfied, 2);
    assert!(!result.compliant);
}

/// A fully compliant report must leave the gate with nothing to flag.
#[test]
fn a_compliant_report_leaves_no_unsatisfied_required_section() {
    let sections = vec![
        required("abstract", "Özet", vec![], 80),
        required("conclusion", "Sonuç", vec![], 80),
    ];
    let doc = document(
        vec![("# 1. ÖZET", 168), ("# 6. SONUÇ", 168)],
        Language::turkish(),
        1200,
    );
    let result = evaluate(7, &template(sections), &doc);
    assert!(result.compliant, "{}", result.summary);
    assert_eq!(
        result
            .sections
            .iter()
            .filter(|finding| finding.required && !finding.is_satisfied())
            .count(),
        0
    );
}

/// The brief's heading-and-content check has two halves. The structural half —
/// does the heading exist, is there enough after it — passed a report whose
/// "Yöntem" section was padded with budget prose, because it only ever counted
/// words. This is the other half.
#[test]
fn a_section_padded_with_unrelated_prose_is_reported_as_off_topic() {
    let sections = vec![required("methodology", "Yöntem", vec!["Metodoloji"], 40)];
    let mut doc = document(vec![("YÖNTEM", 60)], Language::turkish(), 1200);
    doc.sections[0].content = "Proje bütçesi kalemler halinde planlanmış ve ilgili birim \
         tarafından onaylanmıştır. Harcamalar dönemsel olarak raporlanmakta, ekip üyelerinin \
         görev dağılımı haftalık toplantılarda güncellenmektedir. Paydaş iletişimi düzenli \
         yürütülmüş, takvim üzerinde herhangi bir sapma yaşanmamıştır."
        .into();

    let result = evaluate(7, &template(sections), &doc);
    assert_eq!(result.sections[0].status, "off_topic");
    assert!(!result.compliant);
    assert!(
        result.summary.contains("başlığıyla örtüşmüyor"),
        "unexpected summary: {}",
        result.summary
    );
}

#[test]
fn a_section_that_discusses_its_topic_still_passes() {
    let sections = vec![required("methodology", "Yöntem", vec!["Metodoloji"], 40)];
    let mut doc = document(vec![("YÖNTEM", 60)], Language::turkish(), 1200);
    doc.sections[0].content = "Uygulanan yöntem üç aşamadan oluşmaktadır. Veri toplama \
         aşamasında iki tarlada altı ay boyunca ölçüm yapılmış, ardından aykırı değerler \
         ayıklanmıştır. Model eğitimi için veri kümesi üç parçaya bölünmüş ve başarım \
         ortalama mutlak hata ile raporlanmıştır."
        .into();

    let result = evaluate(7, &template(sections), &doc);
    assert_eq!(result.sections[0].status, "present");
    assert!(result.compliant, "{}", result.summary);
}

/// The topic check is bilingual. A section the organisers titled in English —
/// matched to the Turkish heading through the alias the template provides —
/// must still be recognised as on-topic when its body is Turkish, or every
/// English-titled section of every Turkish report would be called off-topic.
#[test]
fn the_topic_check_recognises_turkish_content_under_an_english_title() {
    let sections = vec![required("methodology", "Methodology", vec!["Yöntem"], 40)];
    let mut doc = document(vec![("Yöntem", 60)], Language::turkish(), 1200);
    doc.sections[0].content = "Uygulanan yöntem üç aşamadan oluşmaktadır ve doğrulama \
         kümesinde başarım ölçülmüştür. Veri temizleme adımları betiklenerek yinelenebilir \
         hale getirilmiştir."
        .into();

    let result = evaluate(7, &template(sections), &doc);
    assert_eq!(
        result.sections[0].status, "present",
        "detail: {}",
        result.sections[0].detail
    );
}

/// A short section is already reported as thin; judging its subject on a
/// sentence or two would be guesswork, so the topic check stays out of it.
#[test]
fn a_short_section_is_reported_as_thin_rather_than_off_topic() {
    let sections = vec![required("methodology", "Yöntem", vec![], 100)];
    let mut doc = document(vec![("YÖNTEM", 12)], Language::turkish(), 1200);
    doc.sections[0].content = "Bütçe onaylandı.".into();
    let result = evaluate(7, &template(sections), &doc);
    assert_eq!(result.sections[0].status, "thin");
}

/// A heading recovered from raw text carries no parsed body, so there is
/// nothing to judge and it must not be called off-topic.
#[test]
fn a_heading_without_a_parsed_body_is_never_called_off_topic() {
    let sections = vec![required("methodology", "Yöntem", vec![], 0)];
    let mut doc = document(vec![], Language::turkish(), 1200);
    doc.headings = vec!["YÖNTEM".into()];
    let result = evaluate(7, &template(sections), &doc);
    assert_ne!(result.sections[0].status, "off_topic");
}
