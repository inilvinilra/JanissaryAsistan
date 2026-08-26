use super::*;

#[test]
fn turkish_letters_fold_to_their_ascii_base() {
    assert_eq!(fold_ascii("Güvenlik"), "guvenlik");
    assert_eq!(fold_ascii("SULAMA"), "sulama");
    assert_eq!(fold_ascii("İÇİNDEKİLER"), "icindekiler");
    assert_eq!(fold_ascii("IŞIK"), "isik");
    assert_eq!(fold_ascii("Yapay Zekâ"), "yapay zeka");
}

/// The dotted/dotless I pair is the classic Turkish casing trap: both must land
/// on the same ASCII letter so a keyword matches regardless of how it was typed.
#[test]
fn both_turkish_i_variants_fold_together() {
    assert_eq!(fold_ascii("İ"), fold_ascii("i"));
    assert_eq!(fold_ascii("I"), fold_ascii("ı"));
}

#[test]
fn every_category_in_the_seeded_templates_has_a_vocabulary() {
    for category in [
        "ai",
        "cybersecurity",
        "data-science",
        "edtech",
        "health-tech",
        "ktr",
        "mathematics",
        "odr",
        "physics",
        "robotics",
        "science",
        "software",
        "sustainability",
        "technology",
    ] {
        assert!(
            keywords_for(category).is_some(),
            "{category} için sözlük yok"
        );
    }
}

#[test]
fn vocabularies_carry_both_turkish_and_english_terms() {
    for vocabulary in VOCABULARIES {
        assert!(
            vocabulary.keywords.len() >= 10,
            "{} sözlüğü çok kısa",
            vocabulary.category
        );
        // ASCII-folded storage keeps matching cheap and case-safe.
        for keyword in vocabulary.keywords {
            assert_eq!(
                *keyword,
                fold_ascii(keyword),
                "{} içindeki \"{keyword}\" katlanmamış",
                vocabulary.category
            );
        }
    }
}

#[test]
fn an_unknown_category_has_no_vocabulary() {
    assert!(keywords_for("nonexistent-category").is_none());
}
