use crate::models::{
    Document, ReportTemplate, SectionFinding, TemplateCompliance, TemplateSection,
};

const MATCH_THRESHOLD: f64 = 0.6;

/// Turkish dotted/dotless I does not survive `to_lowercase`, so it is folded explicitly.
fn fold(input: &str) -> String {
    input
        .chars()
        .flat_map(|c| match c {
            'I' => vec!['ı'],
            'İ' => vec!['i'],
            other => other.to_lowercase().collect::<Vec<char>>(),
        })
        .collect()
}

fn is_separator(token: &str) -> bool {
    !token.is_empty()
        && token
            .chars()
            .all(|c| matches!(c, '-' | '–' | '—' | ':' | '.' | ')' | '('))
}

/// How many leading tokens are section numbering rather than title words.
/// Digits always count; roman numerals only when punctuation or a dash follows,
/// so a real word such as "Civil" is never mistaken for "CIVIL".
fn leading_ordinal_tokens(raw: &[&str]) -> usize {
    let Some(first) = raw.first() else { return 0 };
    let core = first.trim_end_matches(['.', ')', '-', ':']);
    if core.is_empty() {
        return 0;
    }
    let had_punctuation = core.len() < first.len();
    let numeric = core.chars().all(|c| c.is_ascii_digit() || c == '.');
    let roman = core.chars().count() <= 5
        && core
            .chars()
            .all(|c| matches!(c, 'i' | 'v' | 'x' | 'l' | 'c' | 'ı'));
    let separator_follows = raw.get(1).is_some_and(|token| is_separator(token));

    if numeric || (roman && (had_punctuation || separator_follows)) {
        if separator_follows { 2 } else { 1 }
    } else {
        0
    }
}

pub fn normalize_heading(input: &str) -> String {
    let folded = fold(input);
    // Markdown and outline markers survive extraction and would otherwise be
    // read as the first title word, so they are dropped before numbering.
    let raw: Vec<&str> = folded
        .split_whitespace()
        .skip_while(|token| {
            token
                .chars()
                .all(|c| matches!(c, '#' | '*' | '>' | '=' | '_'))
        })
        .collect();
    let skip = leading_ordinal_tokens(&raw);
    let kept = if skip < raw.len() {
        &raw[skip..]
    } else {
        &raw[..]
    };

    kept.iter()
        .map(|token| {
            token
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
        })
        .filter(|token| !token.is_empty())
        .collect::<Vec<String>>()
        .join(" ")
}

fn tokens(input: &str) -> Vec<String> {
    normalize_heading(input)
        .split_whitespace()
        .filter(|token| token.chars().count() >= 3)
        .map(str::to_string)
        .collect()
}

/// 1.0 for an exact match, then containment, then token overlap. Below the
/// threshold the candidate is not considered a match at all.
fn similarity(candidate: &str, target: &str) -> f64 {
    let a = normalize_heading(candidate);
    let b = normalize_heading(target);
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    if a == b {
        return 1.0;
    }
    let (shorter, longer) = if a.chars().count() <= b.chars().count() {
        (&a, &b)
    } else {
        (&b, &a)
    };
    if shorter.chars().count() >= 4 && longer.contains(shorter.as_str()) {
        return 0.9;
    }

    let left = tokens(&a);
    let right = tokens(&b);
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    let shared = left.iter().filter(|token| right.contains(token)).count();
    if shared == 0 {
        return 0.0;
    }
    shared as f64 / left.len().min(right.len()) as f64
}

fn best_match<'a>(
    requirement: &TemplateSection,
    candidates: &'a [(String, i64)],
) -> Option<(&'a str, i64, f64)> {
    let mut best: Option<(&str, i64, f64)> = None;
    for (heading, words) in candidates {
        let score = std::iter::once(&requirement.title)
            .chain(requirement.aliases.iter())
            .map(|target| similarity(heading, target))
            .fold(0.0_f64, f64::max);
        if score < MATCH_THRESHOLD {
            continue;
        }
        // On an equal-quality match prefer the candidate carrying real content,
        // so a bare heading never shadows the parsed section with the same name.
        let better = match best {
            None => true,
            Some((_, best_words, best_score)) => {
                score > best_score + f64::EPSILON
                    || ((score - best_score).abs() <= f64::EPSILON && *words > best_words)
            }
        };
        if better {
            best = Some((heading.as_str(), *words, score));
        }
    }
    best
}

/// Whether a section's body actually discusses what its heading promises.
///
/// The structural checks above only ask whether a required heading exists and
/// whether enough words follow it. That passes a report whose "Yöntem" heading
/// is followed by budget tables or filler — which is exactly what padding to a
/// word count produces, and exactly what the brief's heading-and-content check
/// is meant to catch.
///
/// The section's own vocabulary comes from the template: its title and the
/// aliases the organisers wrote for it, widened through
/// [`crate::criterion_vocabulary`] so an English template still recognises a
/// Turkish report. A section is judged off-topic only when it is long enough to
/// be judged at all and contains none of that vocabulary — a single hit is
/// enough to give the applicant the benefit of the doubt.
fn discusses_its_topic(requirement: &TemplateSection, content: &str) -> bool {
    let vocabulary: Vec<String> = std::iter::once(requirement.title.clone())
        .chain(requirement.aliases.iter().cloned())
        .flat_map(|phrase| {
            crate::category_taxonomy::fold_ascii(&phrase)
                .split(|character: char| !character.is_alphanumeric())
                .filter(|token| token.chars().count() >= 4)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .flat_map(|token| crate::criterion_vocabulary::expand(&token))
        .collect();
    if vocabulary.is_empty() {
        return true;
    }
    let folded = crate::category_taxonomy::fold_ascii(content);
    vocabulary.iter().any(|term| folded.contains(term.as_str()))
}

/// Sections shorter than this are already reported as "thin"; judging their
/// subject on a sentence or two would be guesswork.
const MIN_WORDS_TO_JUDGE_TOPIC: i64 = 40;

/// The parsed body under a matched heading. Headings recovered from the raw
/// text carry no section of their own, so those yield nothing to judge and the
/// caller leaves them alone.
fn section_content(document: &Document, heading: &str) -> Option<String> {
    document
        .sections
        .iter()
        .find(|section| normalize_heading(&section.title) == normalize_heading(heading))
        .map(|section| section.content.clone())
        .filter(|content| !content.trim().is_empty())
}

pub fn evaluate(
    project_id: i32,
    template: &ReportTemplate,
    document: &Document,
) -> TemplateCompliance {
    let mut candidates: Vec<(String, i64)> = document
        .sections
        .iter()
        .map(|section| (section.title.clone(), section.word_count as i64))
        .collect();
    for heading in &document.headings {
        if !candidates
            .iter()
            .any(|(title, _)| normalize_heading(title) == normalize_heading(heading))
        {
            candidates.push((heading.clone(), 0));
        }
    }

    let mut findings = Vec::with_capacity(template.sections.len());
    let mut earned = 0.0_f64;
    let mut required_total = 0.0_f64;

    for requirement in &template.sections {
        let matched = best_match(requirement, &candidates);
        let (status, matched_heading, word_count, detail) = match matched {
            // Long enough, but the body never mentions what the heading
            // promises. Padding a report to a word count produces exactly this,
            // and the structural checks alone would pass it.
            Some((heading, words, _))
                if words >= requirement.min_words
                    && words >= MIN_WORDS_TO_JUDGE_TOPIC
                    && matches!(
                        section_content(document, heading),
                        Some(ref content) if !discusses_its_topic(requirement, content)
                    ) =>
            {
                (
                    "off_topic",
                    Some(heading.to_string()),
                    words,
                    format!(
                        "\"{heading}\" başlığı bulundu ancak içeriği bu bölümün konusundan bahsetmiyor"
                    ),
                )
            }
            Some((heading, words, _)) if words >= requirement.min_words => (
                "present",
                Some(heading.to_string()),
                words,
                format!("\"{heading}\" başlığı bulundu, {words} kelime"),
            ),
            Some((heading, words, _)) => (
                "thin",
                Some(heading.to_string()),
                words,
                format!(
                    "\"{heading}\" başlığı bulundu ancak {words} kelime içeriyor, beklenen en az {}",
                    requirement.min_words
                ),
            ),
            None => (
                "missing",
                None,
                0,
                format!("\"{}\" başlığı raporda bulunamadı", requirement.title),
            ),
        };

        if requirement.required {
            required_total += 1.0;
            earned += match status {
                "present" => 1.0,
                // A heading with the wrong content is worth no more than a
                // heading with too little: the required material is absent
                // either way.
                "thin" | "off_topic" => 0.5,
                _ => 0.0,
            };
        }

        findings.push(SectionFinding {
            key: requirement.key.clone(),
            title: requirement.title.clone(),
            required: requirement.required,
            status: status.to_string(),
            matched_heading,
            word_count,
            min_words: requirement.min_words,
            detail,
        });
    }

    let section_score = if required_total == 0.0 {
        100.0
    } else {
        (earned / required_total * 100.0 * 10.0).round() / 10.0
    };

    let detected = document.language.name();
    let language_matches =
        template.expected_language == "Any" || template.expected_language == detected;

    let word_count = document.word_count as i64;
    let word_count_within_range = word_count >= template.min_words
        && (template.max_words <= 0 || word_count <= template.max_words);

    let missing_required = findings
        .iter()
        .filter(|finding| finding.required && finding.status == "missing")
        .count();
    let thin_required = findings
        .iter()
        .filter(|finding| finding.required && finding.status == "thin")
        .count();
    let off_topic_required = findings
        .iter()
        .filter(|finding| finding.required && finding.status == "off_topic")
        .count();

    let compliant = missing_required == 0
        && thin_required == 0
        && off_topic_required == 0
        && language_matches
        && word_count_within_range;

    let summary = if compliant {
        format!(
            "Rapor \"{}\" şablonunun {}. sürümüne uygun",
            template.name, template.version
        )
    } else {
        let mut problems = Vec::new();
        if missing_required > 0 {
            problems.push(format!("{missing_required} zorunlu başlık eksik"));
        }
        if thin_required > 0 {
            problems.push(format!("{thin_required} bölüm beklenen içerikten kısa"));
        }
        if off_topic_required > 0 {
            problems.push(format!(
                "{off_topic_required} bölümün içeriği başlığıyla örtüşmüyor"
            ));
        }
        if !language_matches {
            problems.push(format!(
                "rapor dili {detected}, beklenen {}",
                template.expected_language
            ));
        }
        if !word_count_within_range {
            problems.push(format!("kelime sayısı {word_count} sınırların dışında"));
        }
        problems.join("; ")
    };

    TemplateCompliance {
        project_id,
        template_name: template.name.clone(),
        template_version: template.version,
        compliant,
        section_score,
        sections: findings,
        language_expected: template.expected_language.clone(),
        language_detected: detected.to_string(),
        language_matches,
        word_count,
        min_words: template.min_words,
        max_words: template.max_words,
        word_count_within_range,
        summary,
        evaluated_at: chrono::Utc::now().to_rfc3339(),
    }
}

pub fn default_sections() -> Vec<TemplateSection> {
    vec![
        TemplateSection {
            key: "abstract".into(),
            title: "Özet".into(),
            aliases: vec!["Abstract".into(), "Proje Özeti".into()],
            min_words: 80,
            required: true,
        },
        TemplateSection {
            key: "problem".into(),
            title: "Problem Tanımı".into(),
            aliases: vec![
                "Problem".into(),
                "Problem Statement".into(),
                "Sorun Tanımı".into(),
            ],
            min_words: 100,
            required: true,
        },
        TemplateSection {
            key: "solution".into(),
            title: "Çözüm Yaklaşımı".into(),
            aliases: vec!["Çözüm".into(), "Solution".into(), "Yöntem".into()],
            min_words: 150,
            required: true,
        },
        TemplateSection {
            key: "methodology".into(),
            title: "Yöntem".into(),
            aliases: vec!["Metodoloji".into(), "Methodology".into(), "Method".into()],
            min_words: 150,
            required: true,
        },
        TemplateSection {
            key: "originality".into(),
            title: "Özgünlük".into(),
            aliases: vec!["Yenilikçi Yön".into(), "Originality".into()],
            min_words: 80,
            required: true,
        },
        TemplateSection {
            key: "conclusion".into(),
            title: "Sonuç".into(),
            aliases: vec!["Sonuçlar".into(), "Conclusion".into()],
            min_words: 80,
            required: true,
        },
        TemplateSection {
            key: "references".into(),
            title: "Kaynakça".into(),
            aliases: vec!["Referanslar".into(), "References".into()],
            min_words: 0,
            required: false,
        },
    ]
}

#[cfg(test)]
#[path = "template_tests.rs"]
mod tests;
