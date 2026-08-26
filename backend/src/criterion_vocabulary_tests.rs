use super::*;

#[test]
fn an_english_criterion_term_resolves_to_its_turkish_equivalents() {
    let expanded = expand("innovation");
    assert!(expanded.contains(&"innovation".to_string()));
    assert!(expanded.contains(&"yenilikci".to_string()));
    assert!(expanded.contains(&"ozgunluk".to_string()));
}

/// The table is read both ways, so a competition that writes its criteria in
/// Turkish is matched against an English report just as well.
#[test]
fn a_turkish_criterion_term_resolves_to_its_english_counterpart() {
    assert!(expand("yontem").contains(&"methodology".to_string()));
    assert!(expand("guvenlik").contains(&"security".to_string()));
}

/// Organisers edit KPI templates freely, so an unknown term must degrade to
/// itself rather than disappearing.
#[test]
fn an_unknown_term_contributes_only_itself() {
    assert_eq!(expand("blockchain"), vec!["blockchain".to_string()]);
}

/// Matching is by substring so Turkish suffixes still attach, which makes a
/// short entry far too broad: "yeni" would match "yenilenebilir" and "yeniden"
/// in almost any report and manufacture evidence for the wrong criterion.
#[test]
fn expansions_are_long_enough_to_match_safely() {
    for entry in EQUIVALENTS {
        for equivalent in entry.equivalents {
            assert!(
                equivalent.chars().count() >= MIN_EXPANSION_CHARS,
                "\"{equivalent}\" (for \"{}\") is too short to match safely",
                entry.term
            );
        }
    }
}

/// The table is compared against already-folded text, so an entry carrying a
/// Turkish diacritic or a capital would never match anything.
#[test]
fn every_entry_is_stored_ascii_folded_and_lowercase() {
    for entry in EQUIVALENTS {
        for value in std::iter::once(&entry.term).chain(entry.equivalents.iter()) {
            assert_eq!(
                *value,
                crate::category_taxonomy::fold_ascii(value),
                "\"{value}\" is not in folded form"
            );
        }
    }
}

#[test]
fn the_table_has_no_duplicate_terms() {
    let mut seen = std::collections::HashSet::new();
    for entry in EQUIVALENTS {
        assert!(
            seen.insert(entry.term),
            "\"{}\" appears twice in the table",
            entry.term
        );
    }
}

/// The criterion names shipped with the seeded KPI templates are the ones a
/// Turkish submission will actually be measured against, so each must carry at
/// least one Turkish equivalent.
#[test]
fn every_seeded_criterion_name_has_turkish_vocabulary() {
    const SEEDED: &[&str] = &[
        "Accessibility",
        "Analytical Depth",
        "Autonomy",
        "Clarity",
        "Clinical Applicability",
        "Code Quality",
        "Data Quality & Ethics",
        "Environmental Impact",
        "Experimental Validation",
        "Feasibility",
        "Functionality",
        "Hardware Integration",
        "Impact",
        "Innovation",
        "Methodology",
        "Model Performance",
        "Originality",
        "Pedagogical Value",
        "Problem Definition",
        "Rigor",
        "Safety & Compliance",
        "Scientific Rigor",
        "Security Robustness",
        "Solution Originality",
        "Sustainability",
        "System Architecture",
        "Team Readiness",
        "Technical Design Maturity",
        "Theoretical Soundness",
        "Validation Plan",
        "Visualization & Communication",
    ];
    for name in SEEDED {
        let folded = crate::category_taxonomy::fold_ascii(name);
        let covered = folded
            .split(|character: char| !character.is_alphanumeric())
            .filter(|token| token.chars().count() >= 4)
            .any(|token| expand(token).len() > 1);
        assert!(
            covered,
            "no term in \"{name}\" has a Turkish equivalent, so it can never match a Turkish report"
        );
    }
}
