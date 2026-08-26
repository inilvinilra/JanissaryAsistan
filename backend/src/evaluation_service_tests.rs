use super::*;
use crate::models::AiKpiEvaluation;

fn valid() -> UpsertAiEvaluation {
    UpsertAiEvaluation {
        model_version: "heuristic-0.1.0".into(),
        total_score: 72.5,
        confidence: 0.6,
        source_file_version: Some(1),
        kpi_scores: vec![AiKpiEvaluation {
            name: "Su tasarrufu".into(),
            score: 80.0,
            reason: "Ölçülmüş tasarruf sunulmuştur.".into(),
            evidence: vec!["Yüzde otuz sekiz tasarruf ölçülmüştür.".into()],
            confidence: 0.7,
        }],
        strengths: vec!["Ölçüm sunulmuş.".into()],
        weaknesses: vec!["Maliyet ayrıntısı eksik.".into()],
        missing_information: vec!["Bakım maliyeti yok.".into()],
        risks: vec!["Tek sezon verisi.".into()],
        sources: Vec::new(),
        similar_projects: Vec::new(),
    }
}

#[test]
fn a_well_formed_evaluation_passes() {
    assert!(validate(&valid()).is_ok());
}

/// These are the same bounds the public upsert route enforces. Catching a bad
/// value here names the criterion that produced it, instead of surfacing later
/// as an unexplained readiness-gate failure.
#[test]
fn an_out_of_range_total_is_rejected() {
    let mut evaluation = valid();
    evaluation.total_score = 140.0;
    assert!(validate(&evaluation).is_err());
}

#[test]
fn a_non_finite_total_is_rejected() {
    let mut evaluation = valid();
    evaluation.total_score = f64::NAN;
    assert!(validate(&evaluation).is_err());
}

#[test]
fn an_out_of_range_confidence_is_rejected() {
    let mut evaluation = valid();
    evaluation.confidence = 1.4;
    assert!(validate(&evaluation).is_err());
}

#[test]
fn a_criterion_with_an_invalid_score_is_rejected_by_name() {
    let mut evaluation = valid();
    evaluation.kpi_scores[0].score = -5.0;
    let error = validate(&evaluation).expect_err("an invalid criterion score must be rejected");
    assert!(
        error.to_string().contains("Su tasarrufu"),
        "the error should name the criterion, got: {error}"
    );
}

#[test]
fn a_missing_model_version_is_rejected() {
    let mut evaluation = valid();
    evaluation.model_version = "  ".into();
    assert!(validate(&evaluation).is_err());
}
