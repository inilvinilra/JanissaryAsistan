//! Criterion-level evaluation of a submitted report — MVP gate 06.
//!
//! The brief asks for an "AI fourth eye": a judge-facing pre-assessment that
//! scores the report against the competition's own KPI criteria and produces
//! applicant feedback. Two properties matter more than raw scoring quality.
//!
//! First, every claim must be traceable. A criterion score the judge cannot
//! check is worse than no score, so each one carries verbatim sentences from
//! the report itself, and [`ground_evidence`] discards anything that is not
//! actually in the document. That check is what keeps a language model from
//! inventing a quotation.
//!
//! Second, the gate must never become a single point of failure. This module is
//! entirely deterministic and offline: it always produces a complete
//! evaluation, which the language-model layer then refines when it is
//! available. An expired key or a rate limit degrades the explanation, not the
//! competition.

use std::collections::BTreeSet;

use crate::category_taxonomy::fold_ascii;
use crate::models::{AiKpiEvaluation, Document, KpiTemplate, UpsertAiEvaluation};

/// Quoted evidence is trimmed so the judge sees a sentence, not a page.
const MAX_EVIDENCE_CHARS: usize = 240;
const MAX_EVIDENCE_PER_CRITERION: usize = 3;
/// Sentences shorter than this carry no verifiable claim.
const MIN_EVIDENCE_CHARS: usize = 40;

/// Signals gathered by the earlier gates. They do not change a criterion score
/// — those measure the report against the KPI — but they do surface as risks
/// the judge should weigh before accepting the pre-assessment.
#[derive(Debug, Clone, Default)]
pub struct EvaluationContext {
    pub category_mismatch: Option<String>,
    pub high_similarity_with: Option<String>,
    pub missing_sections: Vec<String>,
    pub thin_sections: Vec<String>,
}

pub fn model_version() -> String {
    format!("heuristic-{}", env!("CARGO_PKG_VERSION"))
}

/// Splits on sentence terminators followed by whitespace. Turkish ordinals
/// ("1. Özet") would otherwise break a heading off as its own sentence, so a
/// terminator preceded by a digit does not end a sentence.
pub fn sentences(text: &str) -> Vec<String> {
    let characters: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut start = 0;
    for index in 0..characters.len() {
        let character = characters[index];
        if !matches!(character, '.' | '!' | '?' | '\n') {
            continue;
        }
        let follows_digit = index > 0 && characters[index - 1].is_ascii_digit();
        let ends_here = character == '\n'
            || (!follows_digit
                && characters
                    .get(index + 1)
                    .is_none_or(|next| next.is_whitespace()));
        if !ends_here {
            continue;
        }
        let sentence: String = characters[start..=index].iter().collect();
        let sentence = sentence.trim();
        if !sentence.is_empty() {
            out.push(sentence.to_string());
        }
        start = index + 1;
    }
    if start < characters.len() {
        let tail: String = characters[start..].iter().collect();
        let tail = tail.trim();
        if !tail.is_empty() {
            out.push(tail.to_string());
        }
    }
    out
}

fn terms_of(text: &str) -> BTreeSet<String> {
    fold_ascii(text)
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| token.chars().count() >= 4)
        .map(str::to_string)
        .collect()
}

/// A criterion's own vocabulary, kept in two parts. The title names the
/// criterion and is specific to it; the description is prose and shares generic
/// engineering words ("model", "system") with every other criterion. Treating
/// them alike let a single incidental word qualify a sentence as evidence for a
/// criterion the report never addressed, so [`qualifies_as_evidence`] weighs
/// them differently.
struct CriterionTerms {
    title: BTreeSet<String>,
    description: BTreeSet<String>,
}

impl CriterionTerms {
    fn all(&self) -> BTreeSet<String> {
        self.title.union(&self.description).cloned().collect()
    }
}

fn criterion_terms(kpi: &KpiTemplate) -> CriterionTerms {
    let title = terms_of(&kpi.name);
    let description = terms_of(&kpi.description)
        .difference(&title)
        .cloned()
        .collect();
    CriterionTerms { title, description }
}

/// One word from the criterion's title is enough; description words only count
/// when several appear together, because any one of them may belong to an
/// unrelated sentence.
fn qualifies_as_evidence(sentence: &str, terms: &CriterionTerms) -> Option<usize> {
    let title_hits = matching_terms(sentence, &terms.title);
    let description_hits = matching_terms(sentence, &terms.description);
    if title_hits >= 1 || description_hits >= 2 {
        Some(title_hits * 2 + description_hits)
    } else {
        None
    }
}

fn matching_terms(sentence: &str, terms: &BTreeSet<String>) -> usize {
    let folded = fold_ascii(sentence);
    terms
        .iter()
        .filter(|term| folded.contains(term.as_str()))
        .count()
}

fn shorten(sentence: &str) -> String {
    if sentence.chars().count() <= MAX_EVIDENCE_CHARS {
        return sentence.to_string();
    }
    let clipped: String = sentence.chars().take(MAX_EVIDENCE_CHARS).collect();
    format!("{}…", clipped.trim_end())
}

/// Sentences from the report that actually mention the criterion, densest
/// first. Returning nothing is a meaningful answer: it means the report never
/// addresses this criterion, and the score below reflects that.
fn evidence_for(document: &Document, terms: &CriterionTerms) -> Vec<String> {
    let mut scored: Vec<(usize, String)> = sentences(&document.raw_text)
        .into_iter()
        .filter(|sentence| sentence.chars().count() >= MIN_EVIDENCE_CHARS)
        .filter_map(|sentence| {
            qualifies_as_evidence(&sentence, terms).map(|weight| (weight, sentence))
        })
        .collect();
    scored.sort_by(|left, right| right.0.cmp(&left.0));
    scored.truncate(MAX_EVIDENCE_PER_CRITERION);
    scored
        .into_iter()
        .map(|(_, sentence)| shorten(&sentence))
        .collect()
}

/// True when the quotation really occurs in the report.
///
/// This is the guard against fabricated evidence. Comparison is whitespace- and
/// diacritic-insensitive because extraction collapses line breaks differently
/// than a model reproduces them, and a trailing ellipsis from [`shorten`] is
/// ignored so trimmed quotations still verify.
pub fn is_grounded(document: &Document, quote: &str) -> bool {
    fn normalize(value: &str) -> String {
        fold_ascii(value)
            .chars()
            .filter(|character| character.is_alphanumeric())
            .collect()
    }
    let needle = normalize(quote.trim_end_matches(['…', '.', ' ']));
    if needle.chars().count() < 20 {
        return false;
    }
    normalize(&document.raw_text).contains(&needle)
}

/// Drops quotations that are not in the report and reports how many survived.
/// A criterion whose evidence was invented keeps its reasoning but loses the
/// confidence that evidence would have justified.
pub fn ground_evidence(document: &Document, evidence: Vec<String>) -> (Vec<String>, usize) {
    let total = evidence.len();
    let kept: Vec<String> = evidence
        .into_iter()
        .filter(|quote| is_grounded(document, quote))
        .collect();
    let dropped = total - kept.len();
    (kept, dropped)
}

/// How many criterion scores rest on nothing the judge can check.
///
/// Recomputed from whichever set of scores is actually being stored: the
/// language-model layer usually finds quotations the deterministic pass missed,
/// and carrying the earlier count forward would have left the saved evaluation
/// stating that criteria are unevidenced while showing their evidence.
pub fn unevidenced_criteria_risk(scores: &[AiKpiEvaluation]) -> Option<String> {
    let unevidenced = scores
        .iter()
        .filter(|score| score.evidence.is_empty())
        .count();
    (unevidenced > 0).then(|| {
        format!(
            "{unevidenced} criterion score(s) rest on no quoted evidence and must be confirmed by the judge."
        )
    })
}

/// Report-wide signals every criterion shares: a submission missing its
/// methodology or its references is weaker on every criterion, not just one.
fn document_baseline(document: &Document) -> f64 {
    let mut base = 45.0;
    if document.has_abstract {
        base += 5.0;
    }
    if document.has_methodology {
        base += 8.0;
    }
    if document.has_conclusion {
        base += 5.0;
    }
    if document.has_references {
        base += 7.0;
    }
    if document.word_count >= 800 {
        base += 5.0;
    }
    base
}

/// Whether the evidence contains measurements. A criterion argued with figures
/// is materially better supported than one argued in prose alone, and this is
/// the single strongest signal available without understanding the language.
fn quantified(evidence: &[String]) -> bool {
    evidence.iter().any(|sentence| {
        sentence.chars().any(|character| character.is_ascii_digit())
            && sentence.chars().count() > MIN_EVIDENCE_CHARS
    })
}

fn score_criterion(document: &Document, kpi: &KpiTemplate) -> AiKpiEvaluation {
    let terms = criterion_terms(kpi);
    let evidence = evidence_for(document, &terms);
    let baseline = document_baseline(document);

    // Coverage is the share of the criterion's own vocabulary the report uses.
    // It is the difference between a report that discusses this criterion and
    // one that merely satisfies the template.
    let all_terms = terms.all();
    let coverage = if all_terms.is_empty() {
        0.0
    } else {
        let folded = fold_ascii(&document.raw_text);
        all_terms
            .iter()
            .filter(|term| folded.contains(term.as_str()))
            .count() as f64
            / all_terms.len() as f64
    };

    let evidence_bonus = (evidence.len() as f64) * 4.0;
    let quantified_bonus = if quantified(&evidence) { 6.0 } else { 0.0 };
    let score = (baseline * 0.55 + coverage * 100.0 * 0.45 + evidence_bonus + quantified_bonus)
        .clamp(0.0, 100.0);

    // Confidence tracks how much of this rests on quoted text rather than on
    // whole-report heuristics, so a judge can see which scores to check first.
    let confidence = (0.25 + (evidence.len() as f64 * 0.15) + coverage * 0.25).clamp(0.0, 0.85);

    let reason = if evidence.is_empty() {
        format!(
            "The report contains no sentence addressing \"{}\". The score reflects overall report completeness only and needs manual confirmation.",
            kpi.name
        )
    } else {
        format!(
            "{} sentence(s) address \"{}\" and cover {:.0}% of the criterion's terminology{}.",
            evidence.len(),
            kpi.name,
            coverage * 100.0,
            if quantified(&evidence) {
                ", supported by reported figures"
            } else {
                ", without reported figures"
            }
        )
    };

    AiKpiEvaluation {
        name: kpi.name.clone(),
        score,
        reason,
        evidence,
        confidence,
    }
}

/// Weighted by the competition's own KPI weights, so the total reflects what
/// the organisers said matters rather than a flat average.
fn weighted_total(scores: &[AiKpiEvaluation], kpis: &[KpiTemplate]) -> f64 {
    let total_weight: f64 = kpis.iter().map(|kpi| kpi.weight).sum();
    if total_weight <= 0.0 {
        return 0.0;
    }
    scores
        .iter()
        .zip(kpis.iter())
        .map(|(score, kpi)| score.score * kpi.weight)
        .sum::<f64>()
        / total_weight
}

fn applicant_feedback(
    scores: &[AiKpiEvaluation],
    document: &Document,
    context: &EvaluationContext,
) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let mut strengths = Vec::new();
    let mut weaknesses = Vec::new();
    let mut missing = Vec::new();
    let mut risks = Vec::new();

    for score in scores {
        if score.score >= 72.0 && !score.evidence.is_empty() {
            strengths.push(format!(
                "\"{}\" is argued with supporting statements in the report.",
                score.name
            ));
        }
        if score.score < 60.0 {
            weaknesses.push(format!(
                "\"{}\" is treated only briefly; expand the reasoning and add supporting detail.",
                score.name
            ));
        }
        if score.evidence.is_empty() {
            missing.push(format!(
                "Add a section that addresses \"{}\" explicitly — the report currently says nothing the criterion can be assessed against.",
                score.name
            ));
        } else if !quantified(&score.evidence) {
            missing.push(format!(
                "Support \"{}\" with measurements or test results; it is currently argued in prose alone.",
                score.name
            ));
        }
    }

    if !document.has_methodology {
        missing.push(
            "Describe the method: how the work was carried out and how the results were obtained."
                .into(),
        );
    }
    if !document.has_references {
        missing.push("Add a reference list; no sources are cited.".into());
    }
    for section in &context.missing_sections {
        missing.push(format!("The required \"{section}\" section is missing."));
    }
    for section in &context.thin_sections {
        weaknesses.push(format!(
            "The \"{section}\" section is shorter than the template requires."
        ));
    }

    if let Some(recommended) = &context.category_mismatch {
        risks.push(format!(
            "The content matches the \"{recommended}\" category more closely than the one applied for; confirm the category before evaluating."
        ));
    }
    if let Some(reference) = &context.high_similarity_with {
        risks.push(format!(
            "Substantial vocabulary overlap with {reference}. This is an advisory signal, not a plagiarism finding, and requires human confirmation."
        ));
    }
    if document.word_count < 400 {
        risks.push(format!(
            "At {} words the report may be too short to assess every criterion fairly.",
            document.word_count
        ));
    }
    if let Some(risk) = unevidenced_criteria_risk(scores) {
        risks.push(risk);
    }

    // Every feedback area must carry something: the readiness gate treats an
    // empty area as incomplete applicant feedback, and an applicant told
    // nothing at all has received no feedback either.
    if strengths.is_empty() {
        strengths
            .push("The report follows the expected structure, which makes it assessable.".into());
    }
    if weaknesses.is_empty() {
        weaknesses
            .push("No criterion fell below the expected level in this pre-assessment.".into());
    }
    if missing.is_empty() {
        missing.push("No missing information was detected against the required criteria.".into());
    }
    if risks.is_empty() {
        risks.push("No automatic risk signal was raised for this submission.".into());
    }

    (strengths, weaknesses, missing, risks)
}

/// The deterministic evaluation. Always succeeds, so gate 06 has an answer even
/// with no model service reachable.
pub fn heuristic_evaluation(
    document: &Document,
    kpis: &[KpiTemplate],
    context: &EvaluationContext,
) -> UpsertAiEvaluation {
    let kpi_scores: Vec<AiKpiEvaluation> = kpis
        .iter()
        .map(|kpi| score_criterion(document, kpi))
        .collect();
    let total_score = weighted_total(&kpi_scores, kpis);
    let confidence = if kpi_scores.is_empty() {
        0.0
    } else {
        kpi_scores.iter().map(|score| score.confidence).sum::<f64>() / kpi_scores.len() as f64
    };
    let (strengths, weaknesses, missing_information, risks) =
        applicant_feedback(&kpi_scores, document, context);

    UpsertAiEvaluation {
        model_version: model_version(),
        total_score,
        confidence,
        source_file_version: None,
        kpi_scores,
        strengths,
        weaknesses,
        missing_information,
        risks,
        sources: Vec::new(),
        similar_projects: Vec::new(),
    }
}

#[cfg(test)]
#[path = "evaluation_tests.rs"]
mod tests;
