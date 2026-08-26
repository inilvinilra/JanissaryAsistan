use super::*;
use crate::models::{FileType, Section};

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
    assert!(!by_key("conclusion").is_satisfied(), "kısa bölüm sayılmamalı");
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
