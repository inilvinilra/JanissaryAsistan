//! Turkish equivalents for the vocabulary competition criteria are written in.
//!
//! KPI templates are authored in English ("Innovation", "Feasibility",
//! "Environmental Impact") while submissions are written in Turkish. Matching a
//! criterion against a report by its own words therefore found nothing: the
//! deterministic evaluation returned no evidence for any criterion of any
//! Turkish report, and every score fell back to whole-document heuristics.
//!
//! This is the same gap [`crate::category_taxonomy`] closes for category
//! matching, solved the same way — with vocabulary rather than translation.
//!
//! The mapping is by *term*, not by criterion name, because organisers edit KPI
//! templates freely. An unrecognised term simply contributes itself, so a
//! criterion this list has never seen degrades to the previous behaviour rather
//! than breaking.

/// Terms shorter than this are omitted from expansions. Matching is by
/// substring so that Turkish suffixes still attach ("yenilikçi" inside
/// "yenilikçiliği"), which makes a short entry dangerously broad: "yeni"
/// would match "yenilenebilir" and "yeniden" in almost any report.
const MIN_EXPANSION_CHARS: usize = 5;

/// Keys and values are ASCII-folded and lowercase, matching the form
/// [`crate::category_taxonomy::fold_ascii`] produces on both sides.
pub struct CriterionTerm {
    pub term: &'static str,
    pub equivalents: &'static [&'static str],
}

/// Bidirectional: a Turkish criterion name is matched against an English
/// report by the same table, read the other way.
pub const EQUIVALENTS: &[CriterionTerm] = &[
    // Novelty and originality
    CriterionTerm {
        term: "innovation",
        equivalents: &["yenilik", "yenilikci", "yenilikcilik", "ozgunluk"],
    },
    CriterionTerm {
        term: "innovative",
        equivalents: &["yenilikci", "yenilik"],
    },
    CriterionTerm {
        term: "originality",
        equivalents: &["ozgunluk", "ozgun", "yenilik"],
    },
    CriterionTerm {
        term: "original",
        equivalents: &["ozgun", "ozgunluk"],
    },
    CriterionTerm {
        term: "novelty",
        equivalents: &["yenilik", "ozgunluk"],
    },
    // Viability
    CriterionTerm {
        term: "feasibility",
        equivalents: &[
            "fizibilite",
            "uygulanabilir",
            "uygulanabilirlik",
            "yapilabilirlik",
        ],
    },
    CriterionTerm {
        term: "feasible",
        equivalents: &["uygulanabilir", "yapilabilir"],
    },
    CriterionTerm {
        term: "applicability",
        equivalents: &["uygulanabilir", "uygulanabilirlik", "uygulama"],
    },
    CriterionTerm {
        term: "viability",
        equivalents: &["surdurulebilir", "yasayabilirlik", "uygulanabilir"],
    },
    // Effect
    CriterionTerm {
        term: "impact",
        equivalents: &["etkisi", "etkilesim", "fayda", "katki"],
    },
    CriterionTerm {
        term: "benefit",
        equivalents: &["fayda", "yarar", "katki"],
    },
    CriterionTerm {
        term: "environmental",
        equivalents: &["cevre", "cevresel", "ekolojik"],
    },
    CriterionTerm {
        term: "environment",
        equivalents: &["cevre", "cevresel"],
    },
    CriterionTerm {
        term: "sustainability",
        equivalents: &["surdurulebilir", "surdurulebilirlik"],
    },
    CriterionTerm {
        term: "sustainable",
        equivalents: &["surdurulebilir"],
    },
    // Method and evidence
    CriterionTerm {
        term: "methodology",
        equivalents: &["yontem", "metodoloji", "yontembilim"],
    },
    CriterionTerm {
        term: "method",
        equivalents: &["yontem", "metot", "metodoloji"],
    },
    CriterionTerm {
        term: "experimental",
        equivalents: &["deney", "deneysel", "denemeler"],
    },
    CriterionTerm {
        term: "experiment",
        equivalents: &["deney", "deneysel"],
    },
    CriterionTerm {
        term: "validation",
        equivalents: &["dogrulama", "gecerleme", "sinama"],
    },
    CriterionTerm {
        term: "verification",
        equivalents: &["dogrulama", "gecerleme"],
    },
    CriterionTerm {
        term: "rigor",
        equivalents: &["titizlik", "bilimsel", "kesinlik"],
    },
    CriterionTerm {
        term: "scientific",
        equivalents: &["bilimsel", "bilim"],
    },
    CriterionTerm {
        term: "theoretical",
        equivalents: &["kuramsal", "teorik", "kuram"],
    },
    CriterionTerm {
        term: "soundness",
        equivalents: &["saglamlik", "tutarlilik", "gecerlilik"],
    },
    CriterionTerm {
        term: "analytical",
        equivalents: &["analitik", "analiz", "cozumleme"],
    },
    CriterionTerm {
        term: "analysis",
        equivalents: &["analiz", "cozumleme", "inceleme"],
    },
    CriterionTerm {
        term: "depth",
        equivalents: &["derinlik", "ayrintili", "kapsamli"],
    },
    // Engineering
    CriterionTerm {
        term: "architecture",
        equivalents: &["mimari", "mimarisi", "yapisi"],
    },
    CriterionTerm {
        term: "design",
        equivalents: &["tasarim", "tasarimi"],
    },
    CriterionTerm {
        term: "maturity",
        equivalents: &["olgunluk", "olgunlugu"],
    },
    CriterionTerm {
        term: "hardware",
        equivalents: &["donanim", "donanimi"],
    },
    CriterionTerm {
        term: "integration",
        equivalents: &["entegrasyon", "butunlesme", "birlestirme"],
    },
    CriterionTerm {
        term: "functionality",
        equivalents: &["islevsellik", "islev", "fonksiyon"],
    },
    CriterionTerm {
        term: "performance",
        equivalents: &["basarim", "performans", "verimlilik"],
    },
    CriterionTerm {
        term: "autonomy",
        equivalents: &["otonom", "ozerk", "bagimsiz"],
    },
    CriterionTerm {
        term: "autonomous",
        equivalents: &["otonom", "ozerk"],
    },
    CriterionTerm {
        term: "robustness",
        equivalents: &["dayaniklilik", "saglamlik", "gurbuzluk"],
    },
    CriterionTerm {
        term: "prototype",
        equivalents: &["prototip", "ornek"],
    },
    // Software and data
    CriterionTerm {
        term: "quality",
        equivalents: &["kalite", "nitelik", "kalitesi"],
    },
    CriterionTerm {
        term: "security",
        equivalents: &["guvenlik", "guvenligi", "siber"],
    },
    CriterionTerm {
        term: "safety",
        equivalents: &["guvenlik", "emniyet"],
    },
    CriterionTerm {
        term: "compliance",
        equivalents: &["uygunluk", "mevzuat", "standart"],
    },
    // "etik" is deliberately absent: at four characters it sits inside
    // "etiket"/"etiketleme" (label/labelling), which appear in most data
    // reports and would manufacture ethics evidence. The inflected "etigi"
    // carries the same meaning without the collision.
    CriterionTerm {
        term: "ethics",
        equivalents: &["etigi", "mahremiyet", "gizlilik"],
    },
    CriterionTerm {
        term: "visualization",
        equivalents: &["gorsellestirme", "grafik", "gorsel"],
    },
    CriterionTerm {
        term: "communication",
        equivalents: &["iletisim", "anlatim", "sunum"],
    },
    CriterionTerm {
        term: "accessibility",
        equivalents: &["erisilebilir", "erisilebilirlik", "erisim"],
    },
    CriterionTerm {
        term: "clarity",
        equivalents: &["aciklik", "anlasilir", "anlasilirlik"],
    },
    // Problem framing and delivery
    CriterionTerm {
        term: "problem",
        equivalents: &["problem", "sorun", "sorunu"],
    },
    CriterionTerm {
        term: "definition",
        equivalents: &["tanimi", "tanimlama", "tanim"],
    },
    CriterionTerm {
        term: "solution",
        equivalents: &["cozum", "cozumu", "yaklasim"],
    },
    CriterionTerm {
        term: "clinical",
        equivalents: &["klinik", "tibbi", "hasta"],
    },
    CriterionTerm {
        term: "pedagogical",
        equivalents: &["pedagojik", "egitsel", "ogretim"],
    },
    CriterionTerm {
        term: "readiness",
        equivalents: &["hazirlik", "hazirbulunusluk"],
    },
];

/// The term itself plus any known equivalents, filtered to those long enough to
/// match safely.
pub fn expand(term: &str) -> Vec<String> {
    let mut out = vec![term.to_string()];
    for entry in EQUIVALENTS {
        // Read in both directions so a Turkish criterion name resolves to its
        // English counterpart in an English report.
        if entry.term == term {
            out.extend(
                entry
                    .equivalents
                    .iter()
                    .filter(|value| value.chars().count() >= MIN_EXPANSION_CHARS)
                    .map(|value| value.to_string()),
            );
        } else if entry.equivalents.contains(&term)
            && entry.term.chars().count() >= MIN_EXPANSION_CHARS
        {
            out.push(entry.term.to_string());
        }
    }
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
#[path = "criterion_vocabulary_tests.rs"]
mod tests;
