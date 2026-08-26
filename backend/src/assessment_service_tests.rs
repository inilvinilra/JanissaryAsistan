use super::*;

fn match_with(jaccard: f64, containment: f64) -> ProjectSimilarityMatch {
    ProjectSimilarityMatch {
        project_id: 2,
        project_reference: "PRJ-000002".into(),
        category: "software".into(),
        similarity: jaccard.max(containment),
        jaccard,
        containment,
        matched_terms: Vec::new(),
    }
}

/// Measured against real Turkish reports: two independently written submissions
/// in the same language and subject area land near jaccard 0.34 / containment
/// 0.55, while a near-duplicate lands near 0.85 / 0.93. Judging both figures
/// against a single 0.45 bar flagged the independent pair too, which would have
/// put every submission in a real competition in front of a human.
#[test]
fn independently_written_reports_are_not_flagged() {
    assert!(
        !needs_review(&match_with(0.341, 0.552)),
        "an independent report must not require review"
    );
}

#[test]
fn a_near_duplicate_is_flagged_by_vocabulary_overlap() {
    assert!(needs_review(&match_with(0.853, 0.934)));
}

/// The padding evasion: a short report copied verbatim into a much longer one.
/// Jaccard collapses because the filler inflates the union, so containment is
/// the only measure that still sees it.
#[test]
fn a_copy_buried_in_padding_is_flagged_by_containment() {
    assert!(needs_review(&match_with(0.21, 0.98)));
}

#[test]
fn thresholds_stay_ordered_and_within_range() {
    assert!(JACCARD_REVIEW_THRESHOLD > 0.0 && JACCARD_REVIEW_THRESHOLD < 1.0);
    assert!(CONTAINMENT_REVIEW_THRESHOLD > JACCARD_REVIEW_THRESHOLD);
    assert!(CONTAINMENT_REVIEW_THRESHOLD < 1.0);
}

#[test]
fn match_limit_prevents_unbounded_api_payloads() {
    assert_eq!(MAX_SIMILARITY_MATCHES, 10);
}
