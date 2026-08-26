use super::*;

#[test]
fn review_threshold_requires_human_attention_only_for_material_overlap() {
    assert!(SIMILARITY_REVIEW_THRESHOLD > 0.0);
    assert!(SIMILARITY_REVIEW_THRESHOLD < 1.0);
}

#[test]
fn match_limit_prevents_unbounded_api_payloads() {
    assert_eq!(MAX_SIMILARITY_MATCHES, 10);
}
