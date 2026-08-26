use super::*;
use crate::evaluation::{EvaluationContext, heuristic_evaluation};
use crate::models::{FileType, Language};

const REPORT: &str = "Bu proje tarımsal sulama sistemlerinde su tüketimini azaltmak amacıyla \
     toprak nem sensörleri kullanan bir karar destek sistemi geliştirmektedir. Saha \
     denemelerinde geleneksel yönteme kıyasla yüzde otuz sekiz oranında su tasarrufu \
     ölçülmüştür. Donanım maliyeti ticari muadillerine kıyasla altıda bir seviyesindedir.";

fn document() -> Document {
    Document {
        filename: "rapor.md".into(),
        file_type: FileType::Markdown,
        raw_text: REPORT.into(),
        word_count: REPORT.split_whitespace().count(),
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

fn kpis() -> Vec<KpiTemplate> {
    vec![
        KpiTemplate {
            name: "Su tasarrufu".into(),
            description: "sulama su tüketimi tasarruf ölçüm".into(),
            weight: 60.0,
        },
        KpiTemplate {
            name: "Maliyet".into(),
            description: "donanım maliyet bütçe".into(),
            weight: 40.0,
        },
    ]
}

fn baseline() -> UpsertAiEvaluation {
    heuristic_evaluation(&document(), &kpis(), &EvaluationContext::default())
}

fn criterion(name: &str, score: f64, evidence: Vec<&str>, confidence: f64) -> ModelCriterion {
    ModelCriterion {
        name: name.into(),
        score,
        reason: format!("{name} gerekçesi"),
        evidence: evidence.into_iter().map(str::to_string).collect(),
        confidence,
    }
}

fn model(criteria: Vec<ModelCriterion>) -> ModelEvaluation {
    ModelEvaluation {
        criteria,
        strengths: vec!["Ölçülmüş su tasarrufu sunulmuştur.".into()],
        weaknesses: vec!["Maliyet analizi ayrıntılandırılmalıdır.".into()],
        missing_information: vec!["Bakım maliyeti belirtilmemiş.".into()],
        risks: vec!["Tek sezonluk saha verisi.".into()],
    }
}

/// The whole point of the grounding check: a model that invents a supporting
/// sentence must not be able to put it in front of a judge.
#[test]
fn a_fabricated_quotation_is_removed_before_the_judge_sees_it() {
    let document = document();
    let merged = merge(
        &document,
        &baseline(),
        model(vec![criterion(
            "Su tasarrufu",
            95.0,
            vec!["Sistem kırk ülkede bağımsız kurumlarca doğrulanmıştır."],
            0.95,
        )]),
        "mistral:test".into(),
        &kpis(),
    );
    let scored = &merged.kpi_scores[0];
    assert!(
        scored.evidence.is_empty(),
        "invented evidence must not survive: {:?}",
        scored.evidence
    );
    assert!(
        scored.confidence <= 0.4,
        "a criterion left without evidence must not stay confident, got {}",
        scored.confidence
    );
}

#[test]
fn a_genuine_quotation_is_kept() {
    let document = document();
    let merged = merge(
        &document,
        &baseline(),
        model(vec![criterion(
            "Su tasarrufu",
            88.0,
            vec![
                "Saha denemelerinde geleneksel yönteme kıyasla yüzde otuz sekiz oranında su tasarrufu ölçülmüştür.",
            ],
            0.9,
        )]),
        "mistral:test".into(),
        &kpis(),
    );
    let scored = &merged.kpi_scores[0];
    assert_eq!(scored.evidence.len(), 1);
    assert_eq!(scored.score, 88.0);
    assert!(scored.confidence > 0.5);
}

/// Mixing one real quotation with one invented one is the realistic failure,
/// and the surviving quotation must not lend the score full confidence.
#[test]
fn partly_fabricated_evidence_keeps_the_real_quote_but_lowers_confidence() {
    let document = document();
    let merged = merge(
        &document,
        &baseline(),
        model(vec![criterion(
            "Su tasarrufu",
            90.0,
            vec![
                "Saha denemelerinde geleneksel yönteme kıyasla yüzde otuz sekiz oranında su tasarrufu ölçülmüştür.",
                "Sistem otuz ülkede patentlenmiştir.",
            ],
            0.9,
        )]),
        "mistral:test".into(),
        &kpis(),
    );
    let scored = &merged.kpi_scores[0];
    assert_eq!(scored.evidence.len(), 1);
    assert!(
        scored.confidence < 0.5,
        "confidence should be penalised, got {}",
        scored.confidence
    );
}

/// The competition's template decides the criterion set. A criterion the model
/// invented must not appear, and one it skipped must keep its baseline result.
#[test]
fn the_criterion_set_comes_from_the_template_not_the_model() {
    let document = document();
    let merged = merge(
        &document,
        &baseline(),
        model(vec![
            criterion("Su tasarrufu", 80.0, vec![], 0.7),
            criterion("Pazarlama Stratejisi", 95.0, vec![], 0.9),
        ]),
        "mistral:test".into(),
        &kpis(),
    );
    let names: Vec<&str> = merged
        .kpi_scores
        .iter()
        .map(|score| score.name.as_str())
        .collect();
    assert_eq!(names, vec!["Su tasarrufu", "Maliyet"]);
}

#[test]
fn an_out_of_range_score_is_clamped() {
    let document = document();
    let merged = merge(
        &document,
        &baseline(),
        model(vec![criterion("Su tasarrufu", 480.0, vec![], 4.0)]),
        "mistral:test".into(),
        &kpis(),
    );
    assert!((0.0..=100.0).contains(&merged.kpi_scores[0].score));
    assert!((0.0..=1.0).contains(&merged.kpi_scores[0].confidence));
    assert!((0.0..=100.0).contains(&merged.total_score));
    assert!((0.0..=1.0).contains(&merged.confidence));
}

/// Risks found by the earlier gates are factual findings about the submission,
/// so the model adds to them rather than overwriting them.
#[test]
fn gate_findings_survive_alongside_model_risks() {
    let document = document();
    let context = EvaluationContext {
        high_similarity_with: Some("PRJ-000042".into()),
        ..EvaluationContext::default()
    };
    let baseline = heuristic_evaluation(&document, &kpis(), &context);
    let merged = merge(
        &document,
        &baseline,
        model(vec![criterion("Su tasarrufu", 80.0, vec![], 0.7)]),
        "mistral:test".into(),
        &kpis(),
    );
    assert!(
        merged.risks.iter().any(|risk| risk.contains("PRJ-000042")),
        "similarity finding was lost: {:?}",
        merged.risks
    );
    assert!(
        merged
            .risks
            .iter()
            .any(|risk| risk.contains("Tek sezonluk saha verisi")),
        "model risk was lost: {:?}",
        merged.risks
    );
}

/// An empty list from the model means it had nothing to add, not that the
/// applicant should be shown nothing.
#[test]
fn empty_model_feedback_falls_back_to_the_deterministic_text() {
    let document = document();
    let merged = merge(
        &document,
        &baseline(),
        ModelEvaluation {
            criteria: vec![criterion("Su tasarrufu", 80.0, vec![], 0.7)],
            strengths: Vec::new(),
            weaknesses: Vec::new(),
            missing_information: Vec::new(),
            risks: Vec::new(),
        },
        "mistral:test".into(),
        &kpis(),
    );
    assert!(!merged.strengths.is_empty());
    assert!(!merged.weaknesses.is_empty());
    assert!(!merged.missing_information.is_empty());
    assert!(!merged.risks.is_empty());
}

#[test]
fn json_is_recovered_from_a_fenced_reply() {
    let content = "Here is the result:\n```json\n{\"criteria\":[]}\n```\nHope that helps.";
    assert_eq!(extract_json(content), Some("{\"criteria\":[]}"));
}

#[test]
fn a_reply_without_json_is_rejected() {
    assert_eq!(extract_json("I cannot assess this report."), None);
}

/// The total must follow the competition's weights, not a flat average, so a
/// model score on a heavily weighted criterion moves it more.
#[test]
fn the_total_follows_the_configured_weights() {
    let document = document();
    let merged = merge(
        &document,
        &baseline(),
        model(vec![
            criterion("Su tasarrufu", 100.0, vec![], 0.8),
            criterion("Maliyet", 0.0, vec![], 0.8),
        ]),
        "mistral:test".into(),
        &kpis(),
    );
    assert!(
        (merged.total_score - 60.0).abs() < 0.001,
        "expected 60 from a 60/40 split, got {}",
        merged.total_score
    );
}

/// The deterministic pass often finds no quotation where the model does — a
/// criterion named in English cannot be matched against a Turkish report, which
/// is exactly the gap the model closes. The stored evaluation must not keep
/// warning that criteria are unevidenced while displaying their evidence.
#[test]
fn the_unevidenced_criteria_warning_is_recomputed_after_merging() {
    let document = document();
    // English criterion names the Turkish report cannot match lexically.
    let english_kpis = vec![
        KpiTemplate {
            name: "Innovation".into(),
            description: "novelty differentiation".into(),
            weight: 60.0,
        },
        KpiTemplate {
            name: "Feasibility".into(),
            description: "deployment cost viability".into(),
            weight: 40.0,
        },
    ];
    let baseline = heuristic_evaluation(&document, &english_kpis, &EvaluationContext::default());
    assert!(
        baseline
            .risks
            .iter()
            .any(|risk| risk.contains("no quoted evidence")),
        "the deterministic pass should leave these criteria unevidenced: {:?}",
        baseline.risks
    );

    let quote = "Saha denemelerinde geleneksel yönteme kıyasla yüzde otuz sekiz oranında su tasarrufu ölçülmüştür.";
    let merged = merge(
        &document,
        &baseline,
        model(vec![
            criterion("Innovation", 88.0, vec![quote], 0.9),
            criterion("Feasibility", 70.0, vec![quote], 0.8),
        ]),
        "mistral:test".into(),
        &english_kpis,
    );

    assert!(
        merged
            .kpi_scores
            .iter()
            .all(|score| !score.evidence.is_empty()),
        "both criteria should carry verified evidence"
    );
    assert!(
        !merged
            .risks
            .iter()
            .any(|risk| risk.contains("no quoted evidence")),
        "a stale warning contradicts the stored evidence: {:?}",
        merged.risks
    );
}

/// When the model genuinely leaves a criterion unevidenced the warning must
/// still appear, and must count the merged result rather than the baseline.
#[test]
fn a_criterion_still_lacking_evidence_is_reported_after_merging() {
    let document = document();
    let quote = "Saha denemelerinde geleneksel yönteme kıyasla yüzde otuz sekiz oranında su tasarrufu ölçülmüştür.";
    let merged = merge(
        &document,
        &baseline(),
        model(vec![
            criterion("Su tasarrufu", 88.0, vec![quote], 0.9),
            criterion(
                "Maliyet",
                70.0,
                vec!["Bu cümle raporda yer almamaktadır."],
                0.5,
            ),
        ]),
        "mistral:test".into(),
        &kpis(),
    );
    assert!(
        merged
            .risks
            .iter()
            .any(|risk| risk.starts_with("1 criterion score(s)")),
        "expected a count of one, got: {:?}",
        merged.risks
    );
}
