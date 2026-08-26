use super::*;
use crate::models::{FileType, Language, Section};

fn report(text: &str) -> Document {
    Document {
        filename: "rapor.md".into(),
        file_type: FileType::Markdown,
        raw_text: text.into(),
        word_count: text.split_whitespace().count(),
        headings: Vec::new(),
        keywords: Vec::new(),
        references: Vec::new(),
        has_references: true,
        has_abstract: true,
        has_conclusion: true,
        has_methodology: true,
        language: Language::turkish(),
        sections: Vec::new(),
    }
}

fn kpi(name: &str, description: &str, weight: f64) -> KpiTemplate {
    KpiTemplate {
        name: name.into(),
        description: description.into(),
        weight,
    }
}

const IRRIGATION_REPORT: &str = "Bu proje tarımsal sulama sistemlerinde su tüketimini azaltmak \
     amacıyla toprak nem sensörleri kullanan bir karar destek sistemi geliştirmektedir. \
     Saha denemelerinde geleneksel yönteme kıyasla yüzde otuz sekiz oranında su tasarrufu \
     ölçülmüştür. Yöntem olarak gradyan artırma tabanlı bir regresyon modeli eğitilmiş ve \
     doğrulama kümesinde ortalama mutlak hata 2.1 milimetre olarak hesaplanmıştır. \
     Donanım maliyeti ticari muadillerine kıyasla altıda bir seviyesindedir.";

#[test]
fn a_criterion_the_report_discusses_is_scored_with_quoted_evidence() {
    let document = report(IRRIGATION_REPORT);
    let result = heuristic_evaluation(
        &document,
        &[kpi(
            "Su tasarrufu",
            "sulama su tüketimi tasarruf ölçüm",
            100.0,
        )],
        &EvaluationContext::default(),
    );

    let criterion = &result.kpi_scores[0];
    assert!(
        !criterion.evidence.is_empty(),
        "a discussed criterion must quote the report"
    );
    assert!(
        criterion
            .evidence
            .iter()
            .all(|quote| is_grounded(&document, quote)),
        "every quotation must come from the report itself"
    );
    assert!(criterion.score > 60.0, "score was {}", criterion.score);
}

/// A criterion the report never addresses must not be quietly awarded a middling
/// score: the judge has to be able to see that nothing supports it.
#[test]
fn a_criterion_the_report_ignores_carries_no_evidence_and_low_confidence() {
    let document = report(IRRIGATION_REPORT);
    let result = heuristic_evaluation(
        &document,
        &[kpi(
            "Ticarileşme planı",
            "pazarlama gelir modeli müşteri segmenti franchise",
            100.0,
        )],
        &EvaluationContext::default(),
    );

    let criterion = &result.kpi_scores[0];
    assert!(criterion.evidence.is_empty());
    assert!(
        criterion.confidence < 0.45,
        "confidence was {}",
        criterion.confidence
    );
    assert!(
        result
            .missing_information
            .iter()
            .any(|item| item.contains("Ticarileşme planı")),
        "the applicant must be told the criterion is unaddressed"
    );
}

/// The guard against fabricated quotations: this is what stops a language model
/// from inventing supporting text the judge would then trust.
#[test]
fn invented_quotations_are_rejected() {
    let document = report(IRRIGATION_REPORT);
    assert!(is_grounded(
        &document,
        "Saha denemelerinde geleneksel yönteme kıyasla yüzde otuz sekiz oranında su tasarrufu ölçülmüştür."
    ));
    assert!(
        !is_grounded(
            &document,
            "Sistem üç yıl boyunca kırk farklı ülkede test edilmiş ve tam verim sağlamıştır."
        ),
        "a sentence absent from the report must not verify"
    );
}

#[test]
fn grounding_reports_how_many_quotations_were_discarded() {
    let document = report(IRRIGATION_REPORT);
    let (kept, dropped) = ground_evidence(
        &document,
        vec![
            "Donanım maliyeti ticari muadillerine kıyasla altıda bir seviyesindedir.".into(),
            "Proje Avrupa Birliği tarafından tamamen finanse edilmiştir.".into(),
        ],
    );
    assert_eq!(kept.len(), 1);
    assert_eq!(dropped, 1);
}

/// Quotations are trimmed for display, so verification has to survive the
/// ellipsis the trimming adds.
#[test]
fn a_trimmed_quotation_still_verifies() {
    let document = report(IRRIGATION_REPORT);
    let trimmed = "Bu proje tarımsal sulama sistemlerinde su tüketimini azaltmak amacıyla…";
    assert!(is_grounded(&document, trimmed));
}

#[test]
fn a_fragment_too_short_to_identify_is_not_accepted_as_evidence() {
    let document = report(IRRIGATION_REPORT);
    assert!(!is_grounded(&document, "sulama"));
}

/// The competition's own weights decide the total, not a flat average.
#[test]
fn the_total_follows_the_configured_kpi_weights() {
    let document = report(IRRIGATION_REPORT);
    let heavy_on_discussed = heuristic_evaluation(
        &document,
        &[
            kpi("Su tasarrufu", "sulama su tüketimi tasarruf ölçüm", 90.0),
            kpi("Ticarileşme", "pazarlama gelir modeli franchise", 10.0),
        ],
        &EvaluationContext::default(),
    );
    let heavy_on_ignored = heuristic_evaluation(
        &document,
        &[
            kpi("Su tasarrufu", "sulama su tüketimi tasarruf ölçüm", 10.0),
            kpi("Ticarileşme", "pazarlama gelir modeli franchise", 90.0),
        ],
        &EvaluationContext::default(),
    );
    assert!(
        heavy_on_discussed.total_score > heavy_on_ignored.total_score,
        "{} should exceed {}",
        heavy_on_discussed.total_score,
        heavy_on_ignored.total_score
    );
}

/// The readiness gate treats an empty feedback area as incomplete applicant
/// feedback, so a clean report must still produce something in every area.
#[test]
fn every_applicant_feedback_area_is_populated() {
    let result = heuristic_evaluation(
        &report(IRRIGATION_REPORT),
        &[kpi("Su tasarrufu", "sulama su tüketimi tasarruf", 100.0)],
        &EvaluationContext::default(),
    );
    assert!(!result.strengths.is_empty());
    assert!(!result.weaknesses.is_empty());
    assert!(!result.missing_information.is_empty());
    assert!(!result.risks.is_empty());
}

/// Findings from the earlier gates reach the judge as risks rather than
/// silently altering a criterion score.
#[test]
fn earlier_gate_findings_are_surfaced_as_risks() {
    let context = EvaluationContext {
        category_mismatch: Some("robotics".into()),
        high_similarity_with: Some("PRJ-000042".into()),
        missing_sections: vec!["Özgünlük".into()],
        thin_sections: vec!["Sonuç".into()],
    };
    let result = heuristic_evaluation(
        &report(IRRIGATION_REPORT),
        &[kpi("Su tasarrufu", "sulama su tüketimi", 100.0)],
        &context,
    );
    assert!(result.risks.iter().any(|risk| risk.contains("robotics")));
    assert!(result.risks.iter().any(|risk| risk.contains("PRJ-000042")));
    assert!(
        result
            .missing_information
            .iter()
            .any(|item| item.contains("Özgünlük"))
    );
    assert!(result.weaknesses.iter().any(|item| item.contains("Sonuç")));
}

/// A similarity finding is advisory. Stating it as a plagiarism determination
/// would put the system in the place the brief reserves for the judge.
#[test]
fn a_similarity_risk_is_worded_as_advisory() {
    let context = EvaluationContext {
        high_similarity_with: Some("PRJ-000042".into()),
        ..EvaluationContext::default()
    };
    let result = heuristic_evaluation(
        &report(IRRIGATION_REPORT),
        &[kpi("Özgünlük", "özgün yenilikçi", 100.0)],
        &context,
    );
    let risk = result
        .risks
        .iter()
        .find(|risk| risk.contains("PRJ-000042"))
        .expect("similarity risk should be present");
    assert!(risk.contains("not a plagiarism finding"));
}

#[test]
fn turkish_ordinals_do_not_split_a_heading_off_as_a_sentence() {
    let parts = sentences("1. Özet Bu bölümde projenin amacı anlatılmaktadır. Sonraki cümle.");
    assert!(
        parts[0].starts_with("1. Özet"),
        "unexpected split: {parts:?}"
    );
}

#[test]
fn scores_and_confidence_stay_inside_their_ranges() {
    let result = heuristic_evaluation(
        &report(IRRIGATION_REPORT),
        &[
            kpi("Su tasarrufu", "sulama su tüketimi tasarruf ölçüm", 50.0),
            kpi("Ticarileşme", "pazarlama gelir modeli", 50.0),
        ],
        &EvaluationContext::default(),
    );
    assert!((0.0..=100.0).contains(&result.total_score));
    assert!((0.0..=1.0).contains(&result.confidence));
    for score in &result.kpi_scores {
        assert!((0.0..=100.0).contains(&score.score), "{}", score.score);
        assert!(
            (0.0..=1.0).contains(&score.confidence),
            "{}",
            score.confidence
        );
    }
}

/// KPI templates are authored in English while submissions are Turkish. Before
/// the criterion vocabulary was introduced this produced no evidence for any
/// criterion of any Turkish report, so every score fell back to whole-document
/// heuristics and the offline path was effectively blind.
#[test]
fn an_english_criterion_matches_a_turkish_report() {
    let document = report(IRRIGATION_REPORT);
    let result = heuristic_evaluation(
        &document,
        // The report describes its method ("Yöntem olarak gradyan artırma…"),
        // which an English criterion can only reach through the vocabulary.
        &[kpi(
            "Methodology",
            "approach and experimental design",
            100.0,
        )],
        &EvaluationContext::default(),
    );
    let criterion = &result.kpi_scores[0];
    assert!(
        !criterion.evidence.is_empty(),
        "an English criterion must reach a Turkish report through the vocabulary"
    );
    assert!(
        criterion
            .evidence
            .iter()
            .all(|quote| is_grounded(&document, quote)),
        "quotations must still come from the report"
    );
}

/// The vocabulary must widen reach without dissolving the distinction between
/// criteria: one the report genuinely never addresses stays unevidenced.
#[test]
fn the_vocabulary_does_not_make_every_criterion_match() {
    let result = heuristic_evaluation(
        &report(IRRIGATION_REPORT),
        &[kpi(
            "Pedagogical Value",
            "curriculum classroom teaching",
            100.0,
        )],
        &EvaluationContext::default(),
    );
    assert!(
        result.kpi_scores[0].evidence.is_empty(),
        "an unrelated criterion must not pick up evidence: {:?}",
        result.kpi_scores[0].evidence
    );
}

/// Reports answer a criterion in the body of the section named after it, not by
/// repeating the criterion's own words. Scanning loose sentences alone missed
/// an entire "Özgünlük" section because its heading was too short to quote and
/// its paragraphs never said "özgünlük" again.
#[test]
fn a_section_named_after_the_criterion_supplies_its_evidence() {
    let mut document = report(IRRIGATION_REPORT);
    document.sections = vec![Section {
        title: "5. Özgünlük".into(),
        content: "Literatürdeki benzer çalışmalar çoğunlukla tek bir sensör tipine dayanmaktadır. \
                  Bu projede sensör verisi ile hava tahmini verisinin birlikte kullanılması ayırt \
                  edici yöndür."
            .into(),
        word_count: 24,
    }];
    document.raw_text = format!("{IRRIGATION_REPORT}\n{}", document.sections[0].content);

    let result = heuristic_evaluation(
        &document,
        &[kpi("Originality", "novelty differentiation", 100.0)],
        &EvaluationContext::default(),
    );
    let criterion = &result.kpi_scores[0];
    assert!(
        !criterion.evidence.is_empty(),
        "the section body should supply the evidence"
    );
    assert!(
        criterion
            .evidence
            .iter()
            .all(|quote| is_grounded(&document, quote)),
        "section evidence must still verify against the report"
    );
}

/// A section that does not name the criterion must not donate its sentences,
/// or every criterion would be evidenced by every section.
#[test]
fn an_unrelated_section_does_not_supply_evidence() {
    let mut document = report(IRRIGATION_REPORT);
    document.sections = vec![Section {
        title: "7. Bütçe".into(),
        content: "Proje bütçesi kalemler halinde planlanmış ve onaylanmıştır. Harcamalar \
                  dönemsel olarak raporlanmaktadır."
            .into(),
        word_count: 16,
    }];
    let result = heuristic_evaluation(
        &document,
        &[kpi(
            "Pedagogical Value",
            "curriculum classroom teaching",
            100.0,
        )],
        &EvaluationContext::default(),
    );
    assert!(result.kpi_scores[0].evidence.is_empty());
}
