use crate::category_taxonomy;
use crate::models::{CategoryTemplate, Document};
use std::collections::BTreeSet;

const MIN_TOKEN_LENGTH: usize = 3;

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryFitResult {
    pub current_category_score: f64,
    pub recommended_category: String,
    pub recommended_category_score: f64,
    pub matched_terms: Vec<String>,
    pub requires_review: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectSimilarityResult {
    /// The stronger of the two measures, shown to the jury as the headline.
    pub similarity: f64,
    pub jaccard: f64,
    pub containment: f64,
    pub matched_terms: Vec<String>,
}

fn tokens(value: &str) -> BTreeSet<String> {
    value
        .to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| token.chars().count() >= MIN_TOKEN_LENGTH)
        .map(str::to_string)
        .collect()
}

fn document_terms(document: &Document) -> BTreeSet<String> {
    let mut terms = tokens(&document.raw_text);
    for keyword in &document.keywords {
        terms.extend(tokens(keyword));
    }
    terms
}

fn overlap_score(left: &BTreeSet<String>, right: &BTreeSet<String>) -> (f64, Vec<String>) {
    if left.is_empty() || right.is_empty() {
        return (0.0, Vec::new());
    }
    let matched_terms = left.intersection(right).cloned().collect::<Vec<_>>();
    let score = matched_terms.len() as f64 / right.len() as f64 * 100.0;
    (score.min(100.0), matched_terms)
}

/// Turkish is agglutinative, so an exact token match misses "sulamada" for the
/// keyword "sulama". A keyword counts when any document token starts with it.
///
/// A short *part* of a multi-word keyword is not required to match, because the
/// tokeniser drops fragments under three characters and "sensor agi" would
/// otherwise never resolve. That exemption must not reach a single-word
/// keyword: applied there it made "ag", "iot", "kod", "web" and "api" match
/// unconditionally, so `software` and `technology` collected free score in
/// every document and pulled correctly-filed cross-domain projects into review.
fn keyword_matches(document_tokens: &BTreeSet<String>, keyword: &str) -> bool {
    let folded = category_taxonomy::fold_ascii(keyword);
    let parts = folded.split_whitespace().collect::<Vec<_>>();
    let multi_word = parts.len() > 1;
    let present = |part: &&str| {
        document_tokens
            .iter()
            .any(|token| token.starts_with(*part) || part.starts_with(token.as_str()))
    };
    parts
        .iter()
        .all(|part| (multi_word && part.chars().count() < 4) || present(part))
}

fn folded_tokens(document: &Document) -> BTreeSet<String> {
    let mut terms = tokens(&category_taxonomy::fold_ascii(&document.raw_text));
    for keyword in &document.keywords {
        terms.extend(tokens(&category_taxonomy::fold_ascii(keyword)));
    }
    terms
}

/// Subject vocabulary is the primary signal; the KPI wording of a category is a
/// weak secondary one, so it only breaks ties.
fn category_evidence(
    document_tokens: &BTreeSet<String>,
    document_terms: &BTreeSet<String>,
    category: &CategoryTemplate,
) -> (f64, Vec<String>) {
    let (kpi_score, mut matched) = overlap_score(document_terms, &category_kpi_terms(category));
    let Some(keywords) = category_taxonomy::keywords_for(&category.category) else {
        return (kpi_score, matched);
    };
    let hits = keywords
        .iter()
        .filter(|keyword| keyword_matches(document_tokens, keyword))
        .copied()
        .collect::<Vec<_>>();
    let vocabulary_score = hits.len() as f64 / keywords.len() as f64 * 100.0;
    matched.splice(0..0, hits.iter().map(|hit| hit.to_string()));
    matched.truncate(25);
    (vocabulary_score * 0.9 + kpi_score * 0.1, matched)
}

fn category_kpi_terms(category: &CategoryTemplate) -> BTreeSet<String> {
    let mut terms = tokens(&category.category);
    for kpi in &category.kpis {
        terms.extend(tokens(&kpi.name));
        terms.extend(tokens(&kpi.description));
    }
    terms
}

pub fn analyze_category_fit(
    document: &Document,
    current_category: &str,
    categories: &[CategoryTemplate],
) -> Option<CategoryFitResult> {
    let document_terms = document_terms(document);
    let document_tokens = folded_tokens(document);
    let mut scored_categories = categories
        .iter()
        .map(|category| {
            let (score, matched_terms) =
                category_evidence(&document_tokens, &document_terms, category);
            (category, score, matched_terms)
        })
        .collect::<Vec<_>>();
    scored_categories.sort_by(|left, right| right.1.total_cmp(&left.1));
    let (recommended, recommended_score, matched_terms) = scored_categories.first()?;
    let current_score = scored_categories
        .iter()
        .find(|(category, _, _)| category.category == current_category)
        .map(|(_, score, _)| *score)
        .unwrap_or(0.0);
    Some(CategoryFitResult {
        current_category_score: current_score,
        recommended_category: recommended.category.clone(),
        recommended_category_score: *recommended_score,
        matched_terms: matched_terms.clone(),
        requires_review: recommended.category != current_category
            && recommended_score - current_score >= 12.0,
    })
}

/// Prefix length used to fold Turkish inflected forms onto a shared bucket
/// ("sulama", "sulamada", "sulamayı" all become "sulam"). Chosen empirically:
/// shorter over-collapses distinct short roots, longer misses common suffix
/// patterns like "-de/-da", "-i/-ı", "-ler/-lar".
const STEM_PREFIX_CHARS: usize = 5;

fn stem(token: &str) -> String {
    token.chars().take(STEM_PREFIX_CHARS).collect()
}

fn stemmed_non_stopwords(text: &str) -> impl Iterator<Item = String> {
    // Stopwords must be matched *before* ASCII-folding: the stopword lists are
    // written with Turkish diacritics ("için"), so folding first ("icin")
    // silently breaks every comparison and the filter never removes anything.
    tokens(text).into_iter().filter_map(|token| {
        let is_stopword = crate::language::TURKISH_STOPWORDS.contains(&token.as_str())
            || crate::language::ENGLISH_STOPWORDS.contains(&token.as_str());
        (!is_stopword).then(|| stem(&category_taxonomy::fold_ascii(&token)))
    })
}

/// Stems of the default report template's own section titles and aliases
/// ("özet", "sonuç", "kaynakça", "abstract", "conclusion", …). Every report
/// that follows the template contains these as headings, so matching on them
/// signals nothing about content overlap — it would make any two compliant
/// reports look artificially similar. Derived from the template itself rather
/// than a hand-kept list, so the two can never drift apart, and cached because
/// a project is compared against every other submission in its competition —
/// rebuilding this per comparison dominated the analysis.
fn boilerplate_stems() -> &'static BTreeSet<String> {
    static STEMS: std::sync::LazyLock<BTreeSet<String>> = std::sync::LazyLock::new(|| {
        crate::template::default_sections()
            .iter()
            .flat_map(|section| {
                std::iter::once(section.title.clone()).chain(section.aliases.clone())
            })
            .flat_map(|phrase| stemmed_non_stopwords(&phrase).collect::<Vec<_>>())
            .collect()
    });
    &STEMS
}

/// Tokens meaningful for comparing two *different* projects' content. Unlike
/// [`document_terms`] — which intentionally keeps stopwords because it is also
/// weighted against short KPI descriptions for category matching — this drops
/// them: two unrelated reports in the same category otherwise share enough
/// "için"/"the"/"ve" noise, or common template headings, to look deceptively
/// similar before any real overlap is counted.
pub fn similarity_tokens(document: &Document) -> BTreeSet<String> {
    let mut terms: BTreeSet<String> = stemmed_non_stopwords(&document.raw_text).collect();
    for keyword in &document.keywords {
        terms.extend(stemmed_non_stopwords(keyword));
    }
    for boilerplate in boilerplate_stems() {
        terms.remove(boilerplate);
    }
    terms
}

/// Takes prepared token sets so a project analysed against every other
/// submission tokenises its own report once instead of once per comparison.
pub fn compare_similarity_tokens(
    left_terms: &BTreeSet<String>,
    right_terms: &BTreeSet<String>,
) -> ProjectSimilarityResult {
    let matched_terms = left_terms
        .intersection(right_terms)
        .cloned()
        .collect::<Vec<_>>();
    if matched_terms.is_empty() {
        return ProjectSimilarityResult {
            similarity: 0.0,
            jaccard: 0.0,
            containment: 0.0,
            matched_terms,
        };
    }
    let union_size = left_terms.union(right_terms).count();
    let jaccard = matched_terms.len() as f64 / union_size as f64;
    // Jaccard alone dilutes toward zero when one report is a short document
    // copied verbatim into a much longer one padded with unrelated filler —
    // the padding inflates the union without adding overlap. The containment
    // coefficient (shared terms over the *smaller* document's own vocabulary)
    // stays high in exactly that case, so a submission cannot dodge detection
    // by burying a copied section under bulk unrelated content.
    //
    // The two are reported separately rather than only as a maximum: for
    // similarly sized reports containment is just Jaccard scaled up by a
    // constant, so judging both against one threshold flagged every report
    // that merely shared a language and a subject area. Each measure carries
    // its own threshold in `assessment_service`.
    let containment = matched_terms.len() as f64 / left_terms.len().min(right_terms.len()) as f64;
    ProjectSimilarityResult {
        similarity: jaccard.max(containment),
        jaccard,
        containment,
        matched_terms,
    }
}

#[cfg(test)]
#[path = "assessment_tests.rs"]
mod tests;
