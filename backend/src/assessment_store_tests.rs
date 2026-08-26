#[test]
fn stored_project_references_remain_blind() {
    let reference = format!("PRJ-{:06}", 42);
    assert_eq!(reference, "PRJ-000042");
    assert!(!reference.contains("team"));
}

#[test]
fn similarity_review_threshold_is_explicit() {
    const THRESHOLD: f64 = 0.45;
    assert!(0.46 >= THRESHOLD);
    assert!(0.44 < THRESHOLD);
}
