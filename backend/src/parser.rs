use crate::models::*;
use anyhow::{Context, Result};
use calamine::{Reader, open_workbook_auto};
use regex::Regex;
use std::{io::Read, path::Path, process::Command};
use unicode_normalization::UnicodeNormalization;

pub fn parse_file(path: &str) -> Result<Document> {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let (raw_text, heading_source) = match ext.as_str() {
        "pdf" => {
            let text = extract_pdf(path)?;
            (text.clone(), text)
        }
        "txt" => {
            let text = extract_txt(path)?;
            (text.clone(), text)
        }
        "md" | "markdown" => extract_markdown(path)?,
        "docx" => {
            let text = extract_docx(path)?;
            (text.clone(), text)
        }
        "xlsx" | "xls" => {
            let text = extract_spreadsheet(path)?;
            (text.clone(), text)
        }
        "csv" => {
            let text = extract_txt(path)?;
            (text.clone(), text)
        }
        "png" | "jpg" | "jpeg" | "webp" => {
            let text = extract_with_ocr(path)?;
            (text.clone(), text)
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported file type: {}. Supported: pdf, txt, md, docx, xlsx, xls, csv, png, jpg, jpeg, webp",
                ext
            ));
        }
    };

    let file_type = match ext.as_str() {
        "pdf" => FileType::Pdf,
        "txt" => FileType::Txt,
        "md" | "markdown" => FileType::Markdown,
        "docx" => FileType::Docx,
        "xlsx" | "xls" | "csv" => FileType::Spreadsheet,
        _ => FileType::Image,
    };

    analyze_text(path, file_type, raw_text, &heading_source)
}

fn extract_pdf(path: &str) -> Result<String> {
    let bytes = std::fs::read(path).context("Could not read PDF file")?;
    let text = pdf_extract::extract_text_from_mem(&bytes).context("Could not extract PDF text")?;
    if text.trim().is_empty() {
        extract_with_ocr(path)
    } else {
        Ok(text)
    }
}

fn extract_txt(path: &str) -> Result<String> {
    std::fs::read_to_string(path).context("Could not read TXT file")
}

fn extract_markdown(path: &str) -> Result<(String, String)> {
    let content = std::fs::read_to_string(path).context("Could not read Markdown file")?;

    let parser = pulldown_cmark::Parser::new(&content);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);

    let tag_re = Regex::new(r"<[^>]+>")?;
    let plain = tag_re.replace_all(&html_output, " ").to_string();

    Ok((plain, content))
}

fn extract_docx(path: &str) -> Result<String> {
    let file = std::fs::File::open(path).context("Could not read DOCX file")?;
    let mut archive = zip::ZipArchive::new(file).context("DOCX is not a valid ZIP document")?;
    let mut document = archive
        .by_name("word/document.xml")
        .context("DOCX document XML is missing")?;
    let mut xml = String::new();
    document
        .read_to_string(&mut xml)
        .context("Could not read DOCX document XML")?;
    let breaks = Regex::new(r"</w:p>|<w:br[^>]*/>|<w:tab[^>]*/>")?;
    let tags = Regex::new(r"<[^>]+>")?;
    Ok(tags
        .replace_all(&breaks.replace_all(&xml, "\n"), " ")
        .to_string())
}

fn extract_spreadsheet(path: &str) -> Result<String> {
    let mut workbook = open_workbook_auto(path).context("Could not open spreadsheet")?;
    let mut output = Vec::new();
    for name in workbook.sheet_names().to_owned() {
        let range = workbook
            .worksheet_range(&name)
            .context("Could not read spreadsheet worksheet")?;
        output.push(format!("Worksheet: {name}"));
        for row in range.rows() {
            let values = row
                .iter()
                .map(ToString::to_string)
                .filter(|value| !value.trim().is_empty())
                .collect::<Vec<_>>();
            if !values.is_empty() {
                output.push(values.join(" | "));
            }
        }
    }
    if output.len() <= 1 {
        anyhow::bail!("Spreadsheet does not contain readable cell text");
    }
    Ok(output.join("\n"))
}

fn extract_with_ocr(path: &str) -> Result<String> {
    let command = std::env::var("TESSERACT_COMMAND").unwrap_or_else(|_| "tesseract".into());
    let languages = std::env::var("OCR_LANGUAGES").unwrap_or_else(|_| "eng".into());
    let output = Command::new(command)
        .arg(path)
        .arg("stdout")
        .arg("-l")
        .arg(languages)
        .output()
        .context("OCR is unavailable; configure TESSERACT_COMMAND")?;
    if !output.status.success() {
        anyhow::bail!(
            "OCR failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let text = String::from_utf8(output.stdout).context("OCR output is not UTF-8")?;
    if text.trim().is_empty() {
        anyhow::bail!("OCR produced no readable text");
    }
    Ok(text)
}

fn analyze_text(
    filename: &str,
    file_type: FileType,
    raw_text: String,
    heading_source: &str,
) -> Result<Document> {
    let normalized = raw_text.nfc().collect::<String>();
    let heading_source_normalized = heading_source.nfc().collect::<String>();
    let lower = normalized.to_lowercase();

    let word_count = normalized.split_whitespace().count();
    let headings = extract_headings(&heading_source_normalized);
    let keywords = extract_keywords(&normalized);
    let references = extract_references(&normalized);
    let language = crate::language::detect(&lower);
    let sections = extract_sections(&heading_source_normalized);

    let has_abstract = lower.contains("özet") || lower.contains("abstract");
    let has_conclusion =
        lower.contains("sonuç") || lower.contains("conclusion") || lower.contains("result");
    let has_methodology =
        lower.contains("yöntem") || lower.contains("method") || lower.contains("methodology");
    let has_references = !references.is_empty()
        || lower.contains("kaynakça")
        || lower.contains("references")
        || lower.contains("bibliography");

    Ok(Document {
        filename: Path::new(filename)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(filename)
            .to_string(),
        file_type,
        raw_text: normalized,
        word_count,
        headings,
        keywords,
        references,
        has_references,
        has_abstract,
        has_conclusion,
        has_methodology,
        language,
        sections,
    })
}

fn extract_headings(text: &str) -> Vec<String> {
    let mut headings = Vec::new();

    let md_re = Regex::new(r"^#{1,6}\s+(.+)$").unwrap();
    let num_re = Regex::new(r"^\d+(\.\d+)*\s+[A-ZÇĞIİÖŞÜ][^.!?]{3,50}$").unwrap();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(cap) = md_re.captures(trimmed) {
            headings.push(cap[1].trim().to_string());
        } else if num_re.is_match(trimmed) {
            headings.push(trimmed.to_string());
        }
    }

    headings.dedup();
    headings
}

pub fn extract_keywords(text: &str) -> Vec<String> {
    let stop_words: std::collections::HashSet<&str> = [
        "ve", "veya", "ile", "için", "bu", "bir", "de", "da", "den", "dan", "en", "çok", "daha",
        "olan", "olarak", "ise", "ancak", "fakat", "the", "a", "an", "and", "or", "in", "on", "at",
        "to", "for", "of", "is", "are", "was", "were", "be", "been", "have", "has", "had", "that",
        "this", "these", "those", "with", "from", "by", "as",
    ]
    .iter()
    .copied()
    .collect();

    let word_re = Regex::new(r"\b[a-zA-ZçğıiöşüÇĞİÖŞÜ]{4,}\b").unwrap();
    let mut freq: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for word in word_re.find_iter(&text.to_lowercase()) {
        let w = word.as_str().to_string();
        if !stop_words.contains(w.as_str()) {
            *freq.entry(w).or_insert(0) += 1;
        }
    }

    let mut sorted: Vec<(String, usize)> = freq.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.into_iter().take(30).map(|(w, _)| w).collect()
}

fn extract_references(text: &str) -> Vec<String> {
    let mut refs = Vec::new();

    let doi_re = Regex::new(r"10\.\d{4,}/[^\s]+").unwrap();
    let url_re = Regex::new(r"https?://[^\s,;)]+").unwrap();
    let bracket_re = Regex::new(r"\[\d+\]").unwrap();

    for doi in doi_re.find_iter(text) {
        refs.push(format!("DOI: {}", doi.as_str()));
    }
    for url in url_re.find_iter(text) {
        refs.push(url.as_str().to_string());
    }
    for br in bracket_re.find_iter(text) {
        refs.push(br.as_str().to_string());
    }

    refs.dedup();
    refs.into_iter().take(50).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_csv_as_a_spreadsheet() {
        let path = std::env::temp_dir().join("jury-parser-test.csv");
        std::fs::write(&path, "Project,Score\nExample,91").expect("fixture should be written");
        let document = parse_file(path.to_str().expect("temporary path should be UTF-8"))
            .expect("CSV should parse");
        assert!(matches!(document.file_type, FileType::Spreadsheet));
        assert!(document.raw_text.contains("Example,91"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn extracts_text_from_docx_document_xml() {
        let path = std::env::temp_dir().join("jury-parser-test.docx");
        let file = std::fs::File::create(&path).expect("fixture should be created");
        let mut archive = zip::ZipWriter::new(file);
        archive
            .start_file(
                "word/document.xml",
                zip::write::SimpleFileOptions::default(),
            )
            .expect("document XML entry should be created");
        use std::io::Write;
        archive
            .write_all(b"<w:document><w:body><w:p><w:r><w:t>Example DOCX project</w:t></w:r></w:p></w:body></w:document>")
            .expect("document XML should be written");
        archive.finish().expect("DOCX archive should finish");
        let document = parse_file(path.to_str().expect("temporary path should be UTF-8"))
            .expect("DOCX should parse");
        assert!(matches!(document.file_type, FileType::Docx));
        assert!(document.raw_text.contains("Example DOCX project"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn extracts_text_from_xlsx_cells() {
        let path = std::env::temp_dir().join("jury-parser-test.xlsx");
        let file = std::fs::File::create(&path).expect("fixture should be created");
        let mut archive = zip::ZipWriter::new(file);
        let entries = [
            (
                "[Content_Types].xml",
                "<?xml version=\"1.0\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/><Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/><Override PartName=\"/xl/worksheets/sheet1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/></Types>",
            ),
            (
                "_rels/.rels",
                "<?xml version=\"1.0\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.xml\"/></Relationships>",
            ),
            (
                "xl/workbook.xml",
                "<?xml version=\"1.0\"?><workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><sheets><sheet name=\"Projects\" sheetId=\"1\" r:id=\"rId1\"/></sheets></workbook>",
            ),
            (
                "xl/_rels/workbook.xml.rels",
                "<?xml version=\"1.0\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet1.xml\"/></Relationships>",
            ),
            (
                "xl/worksheets/sheet1.xml",
                "<?xml version=\"1.0\"?><worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\"><sheetData><row r=\"1\"><c r=\"A1\" t=\"inlineStr\"><is><t>Example XLSX project</t></is></c></row></sheetData></worksheet>",
            ),
        ];
        use std::io::Write;
        for (name, content) in entries {
            archive
                .start_file(name, zip::write::SimpleFileOptions::default())
                .expect("worksheet entry should be created");
            archive
                .write_all(content.as_bytes())
                .expect("worksheet entry should be written");
        }
        archive.finish().expect("XLSX archive should finish");
        let document = parse_file(path.to_str().expect("temporary path should be UTF-8"))
            .expect("XLSX should parse");
        assert!(matches!(document.file_type, FileType::Spreadsheet));
        assert!(document.raw_text.contains("Example XLSX project"));
        let _ = std::fs::remove_file(path);
    }
}

fn extract_sections(text: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let heading_re = Regex::new(r"(?m)^(#{1,3}\s+.+|^\d+\.\s+[A-ZÇĞIİÖŞÜ].+)$").unwrap();

    let positions: Vec<_> = heading_re.find_iter(text).collect();

    for (i, m) in positions.iter().enumerate() {
        let title = m.as_str().trim().to_string();
        let start = m.end();
        let end = if i + 1 < positions.len() {
            positions[i + 1].start()
        } else {
            text.len()
        };

        let content = text[start..end].trim().to_string();
        let word_count = content.split_whitespace().count();

        if !content.is_empty() {
            sections.push(Section {
                title,
                content,
                word_count,
            });
        }
    }

    sections
}
